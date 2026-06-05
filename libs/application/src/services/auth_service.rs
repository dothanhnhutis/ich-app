use crate::dto::auth_dto::{ClientContext, LoginRequest, LoginResponse};
use crate::errors::AppError;
use crate::security::session_token::{SessionToken, hash_token};
use argon2::{self, PasswordVerifier};
use chrono::Duration;
use domain::cache::SessionCache;
use domain::entities::{CachedSession, NewSession, Session, User, UserStatus};
use domain::repositories::{UserRepository, UserSessionRepository};
use uuid::Uuid;
use validator::Validate;

pub struct AuthService<UR, SR, C>
where
    UR: UserRepository,
    SR: UserSessionRepository,
    C: SessionCache,
{
    user_repo: UR,
    session_repo: SR,
    cache: C,
    session_ttl: Duration,
    cache_ttl: Duration,
    db_sync_interval: Duration,
}

impl<UR, SR, C> AuthService<UR, SR, C>
where
    UR: UserRepository,
    SR: UserSessionRepository,
    C: SessionCache,
{
    pub fn new(
        user_repo: UR,
        session_repo: SR,
        cache: C,
        session_ttl: Duration,
        cache_ttl: Duration,
        db_sync_interval: Duration,
    ) -> Self {
        Self {
            user_repo,
            session_repo,
            cache,
            session_ttl,
            cache_ttl,
            db_sync_interval,
        }
    }

    pub async fn login(
        &self,
        request: LoginRequest,
        ctx: ClientContext,
    ) -> Result<LoginResponse, AppError> {
        request
            .validate()
            .map_err(|e| AppError::Validation(e.to_string()))?;

        let user = self
            .user_repo
            .find_by_email(&request.email)
            .await?
            .ok_or_else(|| AppError::Unauthorized("Email hoặc mật khẩu không đúng".into()))?;

        match user.status {
            UserStatus::Active => {}
            UserStatus::PendingPassword => {
                return Err(AppError::Unauthorized(
                    "Tài khoản chưa đặt mật khẩu. Vui lòng kiểm tra email.".into(),
                ));
            }
            UserStatus::Deactivated => {
                return Err(AppError::Unauthorized("Tài khoản đã bị vô hiệu hóa".into()));
            }
        }

        let hash_str = user.password_hash.clone().ok_or_else(|| {
            AppError::Unauthorized("Tài khoản chưa đặt mật khẩu. Vui lòng kiểm tra email.".into())
        })?;

        // 3. Verify password
        let parsed = argon2::PasswordHash::new(&hash_str)
            .map_err(|e| AppError::Internal(format!("Invalid password hash: {}", e)))?;
        argon2::Argon2::default()
            .verify_password(request.password.as_bytes(), &parsed)
            .map_err(|_| AppError::Unauthorized("Email hoặc mật khẩu không đúng".into()))?;

        // 4. Tạo session: sinh token, lưu hash, tính hạn dùng
        let token = SessionToken::generate();
        let expires_at = chrono::Utc::now() + self.session_ttl;

        let new_session = NewSession {
            user_id: user.id,
            token_hash: token.hash,
            device_type: request.device_type,
            device_id: request.device_id,
            device_name: request.device_name,
            platform: request.platform,
            app_version: request.app_version,
            user_agent: ctx.user_agent,
            ip_address: ctx.ip_address,
            expires_at,
        };

        self.session_repo.create(new_session).await?;

        // 5. Return response — trả token THÔ cho client
        Ok(LoginResponse {
            user_id: user.id.to_string(),
            session: token.raw,
            expires_in: self.session_ttl.num_seconds(),
        })
    }

    /// Xác thực một token thô (từ Bearer/cookie) → trả phiên + user.
    /// Cache-first (fail-open), sliding 30 ngày, ghi DB throttle theo `db_sync_interval`.
    pub async fn authenticate(&self, raw_token: &str) -> Result<(Session, User), AppError> {
        let token_hash = hash_token(raw_token);
        let now = chrono::Utc::now();

        // 1. Cache trước; lỗi cache → coi như miss (fail-open), fallback DB.
        let cached = match self.cache.get(&token_hash).await {
            Ok(hit) => hit,
            Err(e) => {
                tracing::warn!("cache get lỗi, fallback DB: {e}");
                None
            }
        };

        let (mut session, user, mut db_synced_at) = match cached {
            Some(c) => (c.session, c.user, c.db_synced_at),
            None => {
                let s = self
                    .session_repo
                    .find_by_token_hash(&token_hash)
                    .await?
                    .ok_or_else(|| AppError::Unauthorized("Phiên đăng nhập không hợp lệ".into()))?;
                let (synced, uid) = (s.updated_at, s.user_id); // đọc trước khi move s
                let u = self
                    .user_repo
                    .find_by_id(uid)
                    .await?
                    .ok_or_else(|| AppError::Unauthorized("Người dùng không tồn tại".into()))?;
                (s, u, synced)
            }
        };

        // 2. Kiểm tra hợp lệ — nếu fail thì dọn cache (fail-open) rồi trả lỗi.
        if session.revoked_at.is_some() {
            let _ = self.cache.remove(&token_hash).await;
            return Err(AppError::Unauthorized(
                "Phiên đăng nhập đã bị thu hồi".into(),
            ));
        }
        if session.expires_at <= now {
            let _ = self.cache.remove(&token_hash).await;
            return Err(AppError::Unauthorized("Phiên đăng nhập đã hết hạn".into()));
        }
        if user.status == UserStatus::Deactivated {
            let _ = self.cache.remove(&token_hash).await;
            return Err(AppError::Unauthorized("Tài khoản đã bị vô hiệu hóa".into()));
        }

        // 3. Sliding: gia hạn 30 ngày.
        let new_expires = now + self.session_ttl;
        session.expires_at = new_expires;

        // 4. Throttle ghi DB; lỗi DB chỉ log (giá trị in-memory đã gia hạn).
        if now - db_synced_at >= self.db_sync_interval {
            match self.session_repo.touch_expires(session.id, new_expires).await {
                Ok(()) => db_synced_at = now,
                Err(e) => tracing::warn!("touch_expires lỗi: {e}"),
            }
        }

        // 5. Refresh cache + gia hạn TTL bản cache (fail-open).
        let entry = CachedSession {
            session: session.clone(),
            user: user.clone(),
            db_synced_at,
        };
        if let Err(e) = self
            .cache
            .put(&token_hash, &entry, self.cache_ttl.num_seconds())
            .await
        {
            tracing::warn!("cache put lỗi: {e}");
        }

        Ok((session, user))
    }

    /// Đăng xuất phiên hiện tại (revoke DB + xóa cache).
    pub async fn logout(&self, session_id: Uuid, token_hash: &str) -> Result<(), AppError> {
        self.session_repo.revoke(session_id, "LOGOUT").await?;
        let _ = self.cache.remove(token_hash).await;
        Ok(())
    }

    /// Đăng xuất tất cả thiết bị của user (revoke DB + xóa cache toàn bộ phiên user).
    pub async fn logout_all(&self, user_id: Uuid) -> Result<(), AppError> {
        self.session_repo
            .revoke_all_for_user(user_id, "FORCED")
            .await?;
        let _ = self.cache.remove_all_for_user(user_id).await;
        Ok(())
    }
}
