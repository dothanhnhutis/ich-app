use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum UserStatus {
    #[serde(rename = "ACTIVE")]
    Active,
    #[serde(rename = "DEACTIVATED")]
    Deactivated,
    #[serde(rename = "PENDING_PASSWORD")]
    PendingPassword,
}

impl UserStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            UserStatus::Active => "ACTIVE",
            UserStatus::Deactivated => "DEACTIVATED",
            UserStatus::PendingPassword => "PENDING_PASSWORD",
        }
    }
}

impl std::str::FromStr for UserStatus {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "ACTIVE" => Ok(UserStatus::Active),
            "DEACTIVATED" => Ok(UserStatus::Deactivated),
            "PENDING_PASSWORD" => Ok(UserStatus::PendingPassword),
            other => Err(format!("Unknown UserStatus: {}", other)),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: Uuid,
    pub email: String,
    pub password_hash: Option<String>,
    pub username: Option<String>,
    pub status: UserStatus,
    pub deactivated_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Dữ liệu để admin tạo user mới (chỉ email; status mặc định PENDING_PASSWORD ở DB).
#[derive(Debug, Clone)]
pub struct NewUser {
    pub email: String,
}

/// Thay đổi cho cập nhật user — chỉ field `Some` mới được ghi.
#[derive(Debug, Clone, Default)]
pub struct UserUpdate {
    pub username: Option<String>,
    pub status: Option<UserStatus>,
}

/// Trường được phép sắp xếp cho danh sách user (whitelist).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UserSortField {
    Email,
    Username,
    Status,
    CreatedAt,
    UpdatedAt,
}

impl std::str::FromStr for UserSortField {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "email" => Ok(UserSortField::Email),
            "username" => Ok(UserSortField::Username),
            "status" => Ok(UserSortField::Status),
            "created_at" => Ok(UserSortField::CreatedAt),
            "updated_at" => Ok(UserSortField::UpdatedAt),
            other => Err(format!("Unknown UserSortField: {}", other)),
        }
    }
}

/// Một tiêu chí sắp xếp (trường + hướng). Dùng lại `SortDir` của entities.
#[derive(Debug, Clone, Copy)]
pub struct UserSort {
    pub field: UserSortField,
    pub dir: crate::entities::SortDir,
}

/// Điều kiện lọc + phân trang + sắp xếp cho danh sách user.
#[derive(Debug, Clone)]
pub struct UserFilter {
    pub email: Option<String>,
    pub username: Option<String>,
    pub status: Option<UserStatus>,
    /// Thứ tự sắp xếp (ưu tiên theo vị trí); rỗng = mặc định created_at DESC.
    pub sort: Vec<UserSort>,
    pub limit: i64,
    pub offset: i64,
}
