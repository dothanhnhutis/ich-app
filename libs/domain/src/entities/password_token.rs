use chrono::{DateTime, Utc};
use uuid::Uuid;

/// Loại token mật khẩu (khớp CHECK constraint của bảng password_tokens).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PasswordTokenType {
    Init,
    ResetPassword,
}

impl PasswordTokenType {
    pub fn as_str(&self) -> &'static str {
        match self {
            PasswordTokenType::Init => "INIT",
            PasswordTokenType::ResetPassword => "RESET-PASSWORD",
        }
    }
}

impl std::str::FromStr for PasswordTokenType {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "INIT" => Ok(PasswordTokenType::Init),
            "RESET-PASSWORD" => Ok(PasswordTokenType::ResetPassword),
            other => Err(format!("Unknown PasswordTokenType: {other}")),
        }
    }
}

#[derive(Debug, Clone)]
pub struct PasswordToken {
    pub id: Uuid,
    pub user_id: Uuid,
    pub token_hash: String,
    pub token_type: PasswordTokenType,
    pub expires_at: DateTime<Utc>,
    pub used_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

/// Dữ liệu để tạo mới một password token.
#[derive(Debug, Clone)]
pub struct NewPasswordToken {
    pub user_id: Uuid,
    pub token_hash: String,
    pub token_type: PasswordTokenType,
    pub expires_at: DateTime<Utc>,
}
