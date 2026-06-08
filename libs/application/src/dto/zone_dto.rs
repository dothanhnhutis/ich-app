use chrono::{DateTime, Utc};
use domain::entities::Zone;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

#[derive(Debug, Serialize)]
pub struct ZoneResponse {
    pub id: String,
    pub location_id: String,
    pub name: String,
    pub zone_type: String,
    pub temp_min_c: Option<f64>,
    pub temp_max_c: Option<f64>,
    pub humidity_max_pct: Option<f64>,
    pub is_light_protected: bool,
    pub is_ventilated: bool,
    pub is_explosion_proof: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<Zone> for ZoneResponse {
    fn from(z: Zone) -> Self {
        Self {
            id: z.id.to_string(),
            location_id: z.location_id.to_string(),
            name: z.name,
            zone_type: z.zone_type.as_str().to_string(),
            temp_min_c: z.temp_min_c,
            temp_max_c: z.temp_max_c,
            humidity_max_pct: z.humidity_max_pct,
            is_light_protected: z.is_light_protected,
            is_ventilated: z.is_ventilated,
            is_explosion_proof: z.is_explosion_proof,
            created_at: z.created_at,
            updated_at: z.updated_at,
        }
    }
}

/// Tạo khu vực mới. `zone_type` được parse + kiểm ở service; range nhiệt độ do DB CHECK chặn.
#[derive(Debug, Deserialize, Validate)]
#[serde(deny_unknown_fields)]
pub struct CreateZoneRequest {
    pub location_id: Uuid,
    #[validate(length(min = 1, max = 150, message = "Tên khu vực 1-150 ký tự"))]
    pub name: String,
    pub zone_type: String,
    pub temp_min_c: Option<f64>,
    pub temp_max_c: Option<f64>,
    #[validate(range(min = 0.0, max = 100.0, message = "Độ ẩm phải trong 0-100"))]
    pub humidity_max_pct: Option<f64>,
    #[serde(default)]
    pub is_light_protected: bool,
    #[serde(default)]
    pub is_ventilated: bool,
    #[serde(default)]
    pub is_explosion_proof: bool,
}

/// Cập nhật khu vực (tất cả tùy chọn). `location_id` cho phép chuyển sang kho khác.
#[derive(Debug, Deserialize, Validate)]
#[serde(deny_unknown_fields)]
pub struct UpdateZoneRequest {
    pub location_id: Option<Uuid>,
    #[validate(length(min = 1, max = 150, message = "Tên khu vực 1-150 ký tự"))]
    pub name: Option<String>,
    pub zone_type: Option<String>,
    pub temp_min_c: Option<f64>,
    pub temp_max_c: Option<f64>,
    #[validate(range(min = 0.0, max = 100.0, message = "Độ ẩm phải trong 0-100"))]
    pub humidity_max_pct: Option<f64>,
    pub is_light_protected: Option<bool>,
    pub is_ventilated: Option<bool>,
    pub is_explosion_proof: Option<bool>,
}

/// Tham số lọc + phân trang + sắp xếp cho GET /zones.
#[derive(Debug, Deserialize)]
pub struct ListZonesQuery {
    pub location_id: Option<Uuid>,
    pub name: Option<String>,
    pub zone_type: Option<String>,
    /// Sắp xếp đa trường: `field:dir,field:dir` (vd `name:asc,created_at:desc`).
    pub sort: Option<String>,
    pub page: Option<u32>,
    pub page_size: Option<u32>,
}
