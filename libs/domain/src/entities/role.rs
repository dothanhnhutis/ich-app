use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Trạng thái vai trò (khớp giá trị cột roles.status: ACTIVE | DEACTIVATED).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RoleStatus {
    #[serde(rename = "ACTIVE")]
    Active,
    #[serde(rename = "DEACTIVATED")]
    Deactivated,
}

impl RoleStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            RoleStatus::Active => "ACTIVE",
            RoleStatus::Deactivated => "DEACTIVATED",
        }
    }
}

impl std::str::FromStr for RoleStatus {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "ACTIVE" => Ok(RoleStatus::Active),
            "DEACTIVATED" => Ok(RoleStatus::Deactivated),
            other => Err(format!("Unknown RoleStatus: {}", other)),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Role {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub status: RoleStatus,
    pub deactivated_at: Option<DateTime<Utc>>,
    pub can_delete: bool,
    pub can_update: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Dữ liệu để tạo vai trò mới (status mặc định ACTIVE ở DB).
#[derive(Debug, Clone)]
pub struct NewRole {
    pub name: String,
    pub description: String,
}

/// Thay đổi cho cập nhật vai trò — chỉ field `Some` mới được ghi.
#[derive(Debug, Clone, Default)]
pub struct RoleUpdate {
    pub name: Option<String>,
    pub description: Option<String>,
    pub status: Option<RoleStatus>,
    /// Some = thay thế toàn bộ tập permission của role; None = không đụng quyền.
    pub permission_ids: Option<Vec<Uuid>>,
}

/// Hướng sắp xếp.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortDir {
    Asc,
    Desc,
}

impl std::str::FromStr for SortDir {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "asc" => Ok(SortDir::Asc),
            "desc" => Ok(SortDir::Desc),
            other => Err(format!("Unknown SortDir: {}", other)),
        }
    }
}

/// Trường được phép sắp xếp cho danh sách vai trò (whitelist).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RoleSortField {
    Name,
    Status,
    CreatedAt,
    UpdatedAt,
}

impl std::str::FromStr for RoleSortField {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "name" => Ok(RoleSortField::Name),
            "status" => Ok(RoleSortField::Status),
            "created_at" => Ok(RoleSortField::CreatedAt),
            "updated_at" => Ok(RoleSortField::UpdatedAt),
            other => Err(format!("Unknown RoleSortField: {}", other)),
        }
    }
}

/// Một tiêu chí sắp xếp (trường + hướng).
#[derive(Debug, Clone, Copy)]
pub struct RoleSort {
    pub field: RoleSortField,
    pub dir: SortDir,
}

/// Điều kiện lọc + phân trang + sắp xếp cho danh sách vai trò.
#[derive(Debug, Clone)]
pub struct RoleFilter {
    pub name: Option<String>,
    pub description: Option<String>,
    pub status: Option<RoleStatus>,
    /// Thứ tự sắp xếp (ưu tiên theo vị trí); rỗng = mặc định created_at DESC.
    pub sort: Vec<RoleSort>,
    pub limit: i64,
    pub offset: i64,
}
