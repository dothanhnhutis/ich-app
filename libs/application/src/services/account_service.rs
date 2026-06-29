use chrono::{Duration, Utc};
use validator::Validate;

use crate::dto::auth_dto::{ForgotPasswordRequest, SetPasswordRequest, SetupAccountRequest};
use crate::errors::AppError;
use crate::ports::EmailPublisher;
use crate::security::password::hash_password;
use crate::security::session_token::{SessionToken, hash_token};
use domain::entities::{NewPasswordToken, PasswordTokenType, UserStatus};
use crate::ports::{PasswordTokenRepository, UserRepository};
use shared::messaging::{EmailJob, ResetPasswordEmail};

pub struct AccountService<UR, PTR, EP>
where
    UR: UserRepository,
    PTR: PasswordTokenRepository,
    EP: EmailPublisher,
{
    user_repo: UR,
    token_repo: PTR,
    email_publisher: EP,
    app_web_url: String,
    reset_token_ttl_secs: i64,
}

impl<UR, PTR, EP> AccountService<UR, PTR, EP>
where
    UR: UserRepository,
    PTR: PasswordTokenRepository,
    EP: EmailPublisher,
{
    pub fn new(
        user_repo: UR,
        token_repo: PTR,
        email_publisher: EP,
        app_web_url: String,
        reset_token_ttl_secs: i64,
    ) -> Self {
        Self {
            user_repo,
            token_repo,
            email_publisher,
            app_web_url,
            reset_token_ttl_secs,
        }
    }

    /// Thiết lập tài khoản từ token INIT (mail admin tạo): đặt username + mật khẩu, kích hoạt.
    pub async fn setup_account(&self, req: SetupAccountRequest) -> Result<(), AppError> {
        req.validate()
            .map_err(|e| AppError::Validation(e.to_string()))?;

        let tok = self.consume_token(&req.token, PasswordTokenType::Init).await?;
        let password_hash = hash_password(&req.password)?;
        self.user_repo
            .activate_account(tok.user_id, req.username.trim(), &password_hash, tok.id)
            .await?;
        Ok(())
    }

    /// Đặt lại mật khẩu từ token RESET-PASSWORD (mail quên mật khẩu).
    pub async fn reset_password(&self, req: SetPasswordRequest) -> Result<(), AppError> {
        req.validate()
            .map_err(|e| AppError::Validation(e.to_string()))?;

        let tok = self
            .consume_token(&req.token, PasswordTokenType::ResetPassword)
            .await?;
        let password_hash = hash_password(&req.password)?;
        self.user_repo
            .reset_password(tok.user_id, &password_hash, tok.id)
            .await?;
        Ok(())
    }

    /// Quên mật khẩu: chỉ gửi mail đặt lại cho user ACTIVE; luôn trả Ok (không lộ email tồn tại).
    pub async fn forgot_password(&self, req: ForgotPasswordRequest) -> Result<(), AppError> {
        req.validate()
            .map_err(|e| AppError::Validation(e.to_string()))?;

        let user = self.user_repo.find_by_email(&req.email).await?;
        if let Some(user) = user
            && user.status == UserStatus::Active
        {
            let token = SessionToken::generate();
            let expires_at = Utc::now() + Duration::seconds(self.reset_token_ttl_secs);
            self.token_repo
                .create(NewPasswordToken {
                    user_id: user.id,
                    token_hash: token.hash,
                    token_type: PasswordTokenType::ResetPassword,
                    expires_at,
                })
                .await?;

            let url = format!("{}/reset-password?token={}", self.app_web_url, token.raw);
            self.email_publisher
                .publish(EmailJob::ResetPassword(ResetPasswordEmail {
                    to: user.email,
                    reset_password_url: url,
                    expires_in_hours: self.reset_token_ttl_secs / 3600,
                }))
                .await?;
        }
        Ok(())
    }

    /// Tìm token còn hiệu lực theo raw token + kiểm đúng loại; lỗi → Validation.
    async fn consume_token(
        &self,
        raw_token: &str,
        expected: PasswordTokenType,
    ) -> Result<domain::entities::PasswordToken, AppError> {
        let token_hash = hash_token(raw_token);
        let tok = self
            .token_repo
            .find_active_by_hash(&token_hash)
            .await?
            .ok_or_else(|| AppError::Validation("Liên kết không hợp lệ hoặc đã hết hạn".into()))?;
        if tok.token_type != expected {
            return Err(AppError::Validation("Liên kết không hợp lệ".into()));
        }
        Ok(tok)
    }
}
