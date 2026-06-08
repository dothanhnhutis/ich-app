use chrono::{DateTime, Utc};
use domain::entities::User;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

#[derive(Debug, Serialize)]
pub struct UserResponse {
    pub id: String,
    pub email: String,
    pub username: Option<String>,
    pub status: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<User> for UserResponse {
    fn from(u: User) -> Self {
        Self {
            id: u.id.to_string(),
            email: u.email,
            username: u.username,
            status: u.status.as_str().to_string(),
            created_at: u.created_at,
            updated_at: u.updated_at,
        }
    }
}

/// Cập nhật user: tên đăng nhập + trạng thái (đều tùy chọn).
#[derive(Debug, Deserialize, Validate)]
#[serde(deny_unknown_fields)]
pub struct UpdateUserRequest {
    #[validate(length(min = 1, max = 100, message = "Tên đăng nhập 1-100 ký tự"))]
    pub username: Option<String>,
    pub status: Option<String>,
}

/// Tham số lọc + phân trang + sắp xếp cho GET /users (từ query string).
#[derive(Debug, Deserialize)]
pub struct ListUsersQuery {
    pub email: Option<String>,
    pub username: Option<String>,
    pub status: Option<String>,
    /// Sắp xếp đa trường: `field:dir,field:dir` (vd `email:asc,created_at:desc`).
    pub sort: Option<String>,
    pub page: Option<u32>,
    pub page_size: Option<u32>,
}

/// Admin tạo user mới: email + danh sách role.
#[derive(Debug, Deserialize, Validate)]
#[serde(deny_unknown_fields)]
pub struct CreateUserRequest {
    #[validate(email(message = "Email không hợp lệ"))]
    pub email: String,
    pub role_ids: Vec<Uuid>,
}

#[derive(Debug, Serialize)]
pub struct CreateUserResponse {
    pub user_id: String,
}
