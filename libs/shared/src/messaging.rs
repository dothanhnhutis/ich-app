use serde::{Deserialize, Serialize};

/// Hợp đồng message gửi qua RabbitMQ giữa `api` (publish) và `worker` (consume).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum EmailJob {
    /// Mail admin tạo tài khoản → link thiết lập tài khoản (username + mật khẩu).
    SetPassword(SetPasswordEmail),
    /// Mail quên mật khẩu → link đặt lại mật khẩu.
    ResetPassword(ResetPasswordEmail),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetPasswordEmail {
    /// Email người nhận.
    pub to: String,
    /// Link thiết lập tài khoản (đã kèm token thô).
    pub set_password_url: String,
    /// Số giờ trước khi link hết hạn (hiển thị trong mail).
    pub expires_in_hours: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResetPasswordEmail {
    /// Email người nhận.
    pub to: String,
    /// Link đặt lại mật khẩu (đã kèm token thô).
    pub reset_password_url: String,
    /// Số giờ trước khi link hết hạn (hiển thị trong mail).
    pub expires_in_hours: i64,
}
