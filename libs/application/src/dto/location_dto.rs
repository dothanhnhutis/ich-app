use chrono::{DateTime, Utc};
use domain::entities::Location;
use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Debug, Serialize)]
pub struct LocationResponse {
    pub id: String,
    pub code: String,
    pub name: String,
    pub address: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<Location> for LocationResponse {
    fn from(l: Location) -> Self {
        Self {
            id: l.id.to_string(),
            code: l.code,
            name: l.name,
            address: l.address,
            created_at: l.created_at,
            updated_at: l.updated_at,
        }
    }
}

/// Tạo kho mới: mã + tên + địa chỉ (tùy chọn).
#[derive(Debug, Deserialize, Validate)]
#[serde(deny_unknown_fields)]
pub struct CreateLocationRequest {
    #[validate(length(min = 1, max = 50, message = "Mã kho 1-50 ký tự"))]
    pub code: String,
    #[validate(length(min = 1, max = 150, message = "Tên kho 1-150 ký tự"))]
    pub name: String,
    #[validate(length(max = 255, message = "Địa chỉ tối đa 255 ký tự"))]
    pub address: Option<String>,
}

/// Cập nhật kho: mã + tên + địa chỉ (đều tùy chọn).
#[derive(Debug, Deserialize, Validate)]
#[serde(deny_unknown_fields)]
pub struct UpdateLocationRequest {
    #[validate(length(min = 1, max = 50, message = "Mã kho 1-50 ký tự"))]
    pub code: Option<String>,
    #[validate(length(min = 1, max = 150, message = "Tên kho 1-150 ký tự"))]
    pub name: Option<String>,
    #[validate(length(max = 255, message = "Địa chỉ tối đa 255 ký tự"))]
    pub address: Option<String>,
}

/// Tham số lọc + phân trang + sắp xếp cho GET /locations (từ query string).
#[derive(Debug, Deserialize)]
pub struct ListLocationsQuery {
    pub code: Option<String>,
    pub name: Option<String>,
    /// Sắp xếp đa trường: `field:dir,field:dir` (vd `code:asc,created_at:desc`).
    pub sort: Option<String>,
    pub page: Option<u32>,
    pub page_size: Option<u32>,
}
