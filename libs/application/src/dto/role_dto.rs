use chrono::{DateTime, Utc};
use domain::entities::Role;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

/// Tạo vai trò mới: name + (description tùy chọn) + danh sách permission không rỗng.
#[derive(Debug, Deserialize, Validate)]
#[serde(deny_unknown_fields)]
pub struct CreateRoleRequest {
    #[validate(length(min = 1, max = 255, message = "Tên vai trò 1-255 ký tự"))]
    pub name: String,
    #[serde(default)]
    pub description: String,
    #[validate(length(min = 1, message = "Phải chọn ít nhất một quyền"))]
    pub permission_ids: Vec<Uuid>,
}

/// Cập nhật một phần vai trò — chỉ field gửi lên mới đổi.
#[derive(Debug, Deserialize, Validate)]
#[serde(deny_unknown_fields)]
pub struct UpdateRoleRequest {
    #[validate(length(min = 1, max = 255, message = "Tên vai trò 1-255 ký tự"))]
    pub name: Option<String>,
    pub description: Option<String>,
    pub status: Option<String>,
    /// Gửi = thay thế toàn bộ tập quyền (≥1); bỏ qua = không đổi quyền.
    #[validate(length(min = 1, message = "Phải chọn ít nhất một quyền"))]
    pub permission_ids: Option<Vec<Uuid>>,
}

/// Tham số lọc + phân trang cho GET /roles (từ query string).
#[derive(Debug, Deserialize)]
pub struct ListRolesQuery {
    pub name: Option<String>,
    pub description: Option<String>,
    pub status: Option<String>,
    /// Sắp xếp đa trường: `field:dir,field:dir` (vd `name:asc,created_at:desc`).
    pub sort: Option<String>,
    pub page: Option<u32>,
    pub page_size: Option<u32>,
}

#[derive(Debug, Serialize)]
pub struct RoleResponse {
    pub id: String,
    pub name: String,
    pub description: String,
    pub status: String,
    pub can_delete: bool,
    pub can_update: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<Role> for RoleResponse {
    fn from(r: Role) -> Self {
        Self {
            id: r.id.to_string(),
            name: r.name,
            description: r.description,
            status: r.status.as_str().to_string(),
            can_delete: r.can_delete,
            can_update: r.can_update,
            created_at: r.created_at,
            updated_at: r.updated_at,
        }
    }
}
