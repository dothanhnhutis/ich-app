use std::str::FromStr;

use chrono::{Duration, Utc};
use uuid::Uuid;
use validator::Validate;

use crate::dto::pagination::Paginated;
use crate::dto::permission_dto::PermissionsResponse;
use crate::dto::role_dto::RoleResponse;
use crate::dto::user_dto::{
    CreateUserRequest, CreateUserResponse, ListUsersQuery, UpdateUserRequest, UserResponse,
};
use crate::errors::AppError;
use crate::ports::EmailPublisher;
use crate::security::session_token::SessionToken;
use domain::entities::{
    NewPasswordToken, NewUser, PasswordTokenType, SortDir, UserFilter, UserSort, UserSortField,
    UserStatus, UserUpdate,
};
use crate::ports::{PasswordTokenRepository, RoleRepository, UserRepository};
use shared::messaging::{EmailJob, SetPasswordEmail};

const DEFAULT_PAGE_SIZE: u32 = 20;
const MAX_PAGE_SIZE: u32 = 100;

/// Parse chuỗi trạng thái → UserStatus, lỗi → Validation.
fn parse_status(s: &str) -> Result<UserStatus, AppError> {
    UserStatus::from_str(s).map_err(|_| AppError::Validation("Trạng thái không hợp lệ".into()))
}

/// Parse trạng thái cho cập nhật — chỉ cho ACTIVE/DEACTIVATED (không cho PENDING_PASSWORD).
fn parse_updatable_status(s: &str) -> Result<UserStatus, AppError> {
    match parse_status(s)? {
        UserStatus::PendingPassword => Err(AppError::Validation(
            "Không thể đặt trạng thái PENDING_PASSWORD".into(),
        )),
        st => Ok(st),
    }
}

/// Chuẩn hoá filter chuỗi: trim, rỗng → None (không lọc).
fn norm(s: Option<String>) -> Option<String> {
    s.map(|v| v.trim().to_string()).filter(|v| !v.is_empty())
}

/// Parse chuỗi sort `field:dir,field:dir` → Vec<UserSort> (thiếu hướng → asc); lỗi → 400.
fn parse_sort(raw: &str) -> Result<Vec<UserSort>, AppError> {
    let mut out = Vec::new();
    for token in raw.split(',') {
        let token = token.trim();
        if token.is_empty() {
            continue;
        }
        let (f, d) = match token.split_once(':') {
            Some((f, d)) => (f.trim(), d.trim()),
            None => (token, "asc"),
        };
        let field = UserSortField::from_str(&f.to_lowercase())
            .map_err(|_| AppError::Validation(format!("Trường sắp xếp không hợp lệ: {f}")))?;
        let dir = SortDir::from_str(&d.to_lowercase())
            .map_err(|_| AppError::Validation(format!("Hướng sắp xếp không hợp lệ: {d}")))?;
        out.push(UserSort { field, dir });
    }
    Ok(out)
}

pub struct UserService<UR, RR, PTR, EP>
where
    UR: UserRepository,
    RR: RoleRepository,
    PTR: PasswordTokenRepository,
    EP: EmailPublisher,
{
    user_repo: UR,
    role_repo: RR,
    token_repo: PTR,
    email_publisher: EP,
    app_web_url: String,
    token_ttl_secs: i64,
}

impl<UR, RR, PTR, EP> UserService<UR, RR, PTR, EP>
where
    UR: UserRepository,
    RR: RoleRepository,
    PTR: PasswordTokenRepository,
    EP: EmailPublisher,
{
    pub fn new(
        user_repo: UR,
        role_repo: RR,
        token_repo: PTR,
        email_publisher: EP,
        app_web_url: String,
        token_ttl_secs: i64,
    ) -> Self {
        Self {
            user_repo,
            role_repo,
            token_repo,
            email_publisher,
            app_web_url,
            token_ttl_secs,
        }
    }

    /// Admin tạo user mới: tạo user + gán role + sinh token INIT + đẩy email vào queue.
    pub async fn create_user(
        &self,
        req: CreateUserRequest,
    ) -> Result<CreateUserResponse, AppError> {
        req.validate()
            .map_err(|e| AppError::Validation(e.to_string()))?;
        if req.role_ids.is_empty() {
            return Err(AppError::Validation("Phải chọn ít nhất một vai trò".into()));
        }

        // 1. Tạo user + gán role (transaction trong repo).
        let user = self
            .user_repo
            .create_with_roles(NewUser { email: req.email }, &req.role_ids)
            .await?;

        // 2. Sinh token INIT + đẩy email "thiết lập tài khoản" vào hàng chờ.
        self.send_setup_email(user.id, &user.email).await?;

        Ok(CreateUserResponse {
            user_id: user.id.to_string(),
        })
    }

    /// Admin gửi lại mail thiết lập cho user CHƯA kích hoạt: vô hiệu token INIT cũ
    /// + cấp token mới (24h) + gửi lại mail.
    pub async fn resend_setup(&self, user_id: Uuid) -> Result<(), AppError> {
        let user = self
            .user_repo
            .find_by_id(user_id)
            .await?
            .ok_or_else(|| AppError::NotFound("Người dùng không tồn tại".into()))?;
        if user.status != UserStatus::PendingPassword {
            return Err(AppError::Validation(
                "Tài khoản đã kích hoạt, không thể gửi lại liên kết".into(),
            ));
        }

        // Vô hiệu token INIT cũ còn hiệu lực → chỉ link mới nhất dùng được.
        self.token_repo
            .invalidate_active(user_id, PasswordTokenType::Init)
            .await?;
        self.send_setup_email(user_id, &user.email).await?;
        Ok(())
    }

    /// Sinh token INIT (raw vào link, hash lưu DB) + publish mail thiết lập tài khoản.
    async fn send_setup_email(&self, user_id: Uuid, email: &str) -> Result<(), AppError> {
        let token = SessionToken::generate();
        let expires_at = Utc::now() + Duration::seconds(self.token_ttl_secs);
        self.token_repo
            .create(NewPasswordToken {
                user_id,
                token_hash: token.hash,
                token_type: PasswordTokenType::Init,
                expires_at,
            })
            .await?;

        let url = format!("{}/setup-account?token={}", self.app_web_url, token.raw);
        self.email_publisher
            .publish(EmailJob::SetPassword(SetPasswordEmail {
                to: email.to_string(),
                set_password_url: url,
                expires_in_hours: self.token_ttl_secs / 3600,
            }))
            .await?;
        Ok(())
    }

    /// Mã permission của một user (cho RBAC).
    pub async fn permission_codes(&self, user_id: Uuid) -> Result<Vec<String>, AppError> {
        self.role_repo.find_permission_codes_for_user(user_id).await
    }

    /// Danh sách user (lọc + phân trang + sắp xếp).
    pub async fn list_users(
        &self,
        q: ListUsersQuery,
    ) -> Result<Paginated<UserResponse>, AppError> {
        let page = q.page.unwrap_or(1).max(1);
        let page_size = q.page_size.unwrap_or(DEFAULT_PAGE_SIZE).clamp(1, MAX_PAGE_SIZE);

        let status = match q.status.as_deref() {
            Some(s) => Some(parse_status(s)?),
            None => None,
        };

        let sort = match q.sort.as_deref() {
            Some(s) => parse_sort(s)?,
            None => Vec::new(),
        };

        let filter = UserFilter {
            email: norm(q.email),
            username: norm(q.username),
            status,
            sort,
            limit: page_size as i64,
            offset: ((page - 1) * page_size) as i64,
        };

        let (users, total) = self.user_repo.list(filter).await?;

        let total_pages = if total == 0 {
            0
        } else {
            ((total as u64).div_ceil(page_size as u64)) as u32
        };

        Ok(Paginated {
            items: users.into_iter().map(UserResponse::from).collect(),
            page,
            page_size,
            total_items: total,
            total_pages,
            has_next: page < total_pages,
            has_prev: page > 1,
        })
    }

    /// Catalog toàn bộ permission, nhóm theo prefix của code (phần trước dấu `_` đầu tiên).
    pub async fn list_permissions_grouped(&self) -> Result<PermissionsResponse, AppError> {
        let perms = self.role_repo.find_all_permissions().await?;
        Ok(PermissionsResponse::group_by_prefix(perms))
    }

    /// Cập nhật username/status của user. status chỉ nhận ACTIVE/DEACTIVATED.
    pub async fn update_user(
        &self,
        id: Uuid,
        req: UpdateUserRequest,
    ) -> Result<UserResponse, AppError> {
        req.validate()
            .map_err(|e| AppError::Validation(e.to_string()))?;

        let status = match req.status.as_deref() {
            Some(s) => Some(parse_updatable_status(s)?),
            None => None,
        };

        let changes = UserUpdate {
            username: req.username.map(|u| u.trim().to_string()),
            status,
        };

        let updated = self
            .user_repo
            .update(id, changes)
            .await?
            .ok_or_else(|| AppError::NotFound("Người dùng không tồn tại".into()))?;
        Ok(UserResponse::from(updated))
    }

    /// Xoá mềm user (404 nếu không tồn tại / đã xoá). Thu hồi phiên do handler điều phối.
    pub async fn delete_user(&self, id: Uuid) -> Result<(), AppError> {
        self.user_repo.soft_delete(id).await?;
        Ok(())
    }

    /// Danh sách role được gán cho user (chỉ role chưa xoá mềm).
    pub async fn user_roles(&self, id: Uuid) -> Result<Vec<RoleResponse>, AppError> {
        self.user_repo
            .find_by_id(id)
            .await?
            .ok_or_else(|| AppError::NotFound("Người dùng không tồn tại".into()))?;
        let roles = self.role_repo.find_roles_for_user(id).await?;
        Ok(roles.into_iter().map(RoleResponse::from).collect())
    }
}
