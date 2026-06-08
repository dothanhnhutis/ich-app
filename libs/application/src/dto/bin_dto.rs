use chrono::{DateTime, Utc};
use domain::entities::Bin;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

#[derive(Debug, Serialize)]
pub struct BinResponse {
    pub id: String,
    pub zone_id: String,
    pub code: String,
    pub name: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<Bin> for BinResponse {
    fn from(b: Bin) -> Self {
        Self {
            id: b.id.to_string(),
            zone_id: b.zone_id.to_string(),
            code: b.code,
            name: b.name,
            created_at: b.created_at,
            updated_at: b.updated_at,
        }
    }
}

/// Tạo kệ mới.
#[derive(Debug, Deserialize, Validate)]
#[serde(deny_unknown_fields)]
pub struct CreateBinRequest {
    pub zone_id: Uuid,
    #[validate(length(min = 1, max = 50, message = "Mã kệ 1-50 ký tự"))]
    pub code: String,
    #[validate(length(min = 1, max = 255, message = "Tên kệ 1-255 ký tự"))]
    pub name: String,
}

/// Cập nhật kệ (tất cả tùy chọn). `zone_id` cho phép chuyển sang khu vực khác.
#[derive(Debug, Deserialize, Validate)]
#[serde(deny_unknown_fields)]
pub struct UpdateBinRequest {
    pub zone_id: Option<Uuid>,
    #[validate(length(min = 1, max = 50, message = "Mã kệ 1-50 ký tự"))]
    pub code: Option<String>,
    #[validate(length(min = 1, max = 255, message = "Tên kệ 1-255 ký tự"))]
    pub name: Option<String>,
}

/// Tham số lọc + phân trang + sắp xếp cho GET /bins.
#[derive(Debug, Deserialize)]
pub struct ListBinsQuery {
    pub zone_id: Option<Uuid>,
    pub code: Option<String>,
    pub name: Option<String>,
    /// Sắp xếp đa trường: `field:dir,field:dir` (vd `code:asc,created_at:desc`).
    pub sort: Option<String>,
    pub page: Option<u32>,
    pub page_size: Option<u32>,
}
