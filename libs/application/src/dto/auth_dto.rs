use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Debug, Deserialize, Validate)]
#[serde(deny_unknown_fields)]
pub struct LoginRequest {
    #[validate(email)]
    pub email: String,

    #[validate(length(min = 8, message = "Email và mật khẩu không hợp lệ."))]
    pub password: String,

    pub app_version: Option<String>,

    pub platform: Option<String>,

    pub device_type: String,

    pub device_name: Option<String>,

    pub device_id: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct LoginResponse {
    pub user_id: String,
    /// Token session THÔ — desktop dùng làm bearer, web app lưu trong cookie.
    pub session: String,
    /// Thời gian sống của session, tính bằng giây.
    pub expires_in: i64,
}

/// Metadata lấy từ tầng HTTP (không nằm trong body request).
#[derive(Debug, Default, Clone)]
pub struct ClientContext {
    pub user_agent: Option<String>,
    pub ip_address: Option<String>,
}

/// Đặt lại mật khẩu từ link trong email quên mật khẩu (token RESET-PASSWORD).
#[derive(Debug, Deserialize, Validate)]
#[serde(deny_unknown_fields)]
pub struct SetPasswordRequest {
    pub token: String,

    #[validate(length(min = 8, message = "Mật khẩu tối thiểu 8 ký tự"))]
    pub password: String,
}

/// Thiết lập tài khoản từ link mail admin tạo (token INIT): nhập username + mật khẩu.
#[derive(Debug, Deserialize, Validate)]
#[serde(deny_unknown_fields)]
pub struct SetupAccountRequest {
    pub token: String,

    #[validate(length(min = 1, max = 100, message = "Tên đăng nhập 1-100 ký tự"))]
    pub username: String,

    #[validate(length(min = 8, message = "Mật khẩu tối thiểu 8 ký tự"))]
    pub password: String,
}

/// Yêu cầu quên mật khẩu: nhập email để nhận link đặt lại.
#[derive(Debug, Deserialize, Validate)]
#[serde(deny_unknown_fields)]
pub struct ForgotPasswordRequest {
    #[validate(email(message = "Email không hợp lệ"))]
    pub email: String,
}
