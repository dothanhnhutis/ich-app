use chrono::{DateTime, Utc};
use domain::entities::Item;
use serde::{Deserialize, Serialize};
use validator::Validate;

/// `is_lot_controlled` mặc định TRUE (khớp default cột DB) khi client không gửi.
fn default_true() -> bool {
    true
}

#[derive(Debug, Serialize)]
pub struct ItemResponse {
    pub id: String,
    pub sku: String,
    pub name: String,
    #[serde(rename = "type")]
    pub item_type: String,
    pub base_uom: String,
    pub packaging_level: Option<String>,
    pub is_purchasable: bool,
    pub is_sellable: bool,
    pub has_bom: bool,
    pub is_lot_controlled: bool,
    pub is_phantom: bool,
    pub density_g_ml: Option<f64>,
    pub shelf_life_days: Option<i32>,
    pub pao_months: Option<i16>,
    pub inci_name: Option<String>,
    pub cas_number: Option<String>,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<Item> for ItemResponse {
    fn from(i: Item) -> Self {
        Self {
            id: i.id.to_string(),
            sku: i.sku,
            name: i.name,
            item_type: i.item_type.as_str().to_string(),
            base_uom: i.base_uom,
            packaging_level: i.packaging_level.map(|p| p.as_str().to_string()),
            is_purchasable: i.is_purchasable,
            is_sellable: i.is_sellable,
            has_bom: i.has_bom,
            is_lot_controlled: i.is_lot_controlled,
            is_phantom: i.is_phantom,
            density_g_ml: i.density_g_ml,
            shelf_life_days: i.shelf_life_days,
            pao_months: i.pao_months,
            inci_name: i.inci_name,
            cas_number: i.cas_number,
            description: i.description,
            created_at: i.created_at,
            updated_at: i.updated_at,
        }
    }
}

/// Tạo vật tư mới. `type`/`packaging_level` parse + kiểm ở service (per-type authz).
#[derive(Debug, Deserialize, Validate)]
#[serde(deny_unknown_fields)]
pub struct CreateItemRequest {
    #[validate(length(min = 1, max = 50, message = "SKU 1-50 ký tự"))]
    pub sku: String,
    #[validate(length(min = 1, max = 255, message = "Tên vật tư 1-255 ký tự"))]
    pub name: String,
    #[serde(rename = "type")]
    pub item_type: String,
    #[validate(length(min = 1, max = 20, message = "Đơn vị cơ sở 1-20 ký tự"))]
    pub base_uom: String,
    pub packaging_level: Option<String>,
    #[serde(default)]
    pub is_purchasable: bool,
    #[serde(default)]
    pub is_sellable: bool,
    #[serde(default)]
    pub has_bom: bool,
    #[serde(default = "default_true")]
    pub is_lot_controlled: bool,
    #[serde(default)]
    pub is_phantom: bool,
    pub density_g_ml: Option<f64>,
    pub shelf_life_days: Option<i32>,
    pub pao_months: Option<i16>,
    #[validate(length(max = 255, message = "Tên INCI tối đa 255 ký tự"))]
    pub inci_name: Option<String>,
    #[validate(length(max = 20, message = "Số CAS tối đa 20 ký tự"))]
    pub cas_number: Option<String>,
    pub description: Option<String>,
}

/// Cập nhật vật tư (tất cả tùy chọn). `type` & `base_uom` **bị khoá** — không nhận ở đây.
#[derive(Debug, Deserialize, Validate)]
#[serde(deny_unknown_fields)]
pub struct UpdateItemRequest {
    #[validate(length(min = 1, max = 50, message = "SKU 1-50 ký tự"))]
    pub sku: Option<String>,
    #[validate(length(min = 1, max = 255, message = "Tên vật tư 1-255 ký tự"))]
    pub name: Option<String>,
    pub packaging_level: Option<String>,
    pub is_purchasable: Option<bool>,
    pub is_sellable: Option<bool>,
    pub has_bom: Option<bool>,
    pub is_lot_controlled: Option<bool>,
    pub is_phantom: Option<bool>,
    pub density_g_ml: Option<f64>,
    pub shelf_life_days: Option<i32>,
    pub pao_months: Option<i16>,
    #[validate(length(max = 255, message = "Tên INCI tối đa 255 ký tự"))]
    pub inci_name: Option<String>,
    #[validate(length(max = 20, message = "Số CAS tối đa 20 ký tự"))]
    pub cas_number: Option<String>,
    pub description: Option<String>,
}

/// Tham số lọc + phân trang + sắp xếp cho GET /items.
#[derive(Debug, Deserialize)]
pub struct ListItemsQuery {
    pub sku: Option<String>,
    pub name: Option<String>,
    #[serde(rename = "type")]
    pub item_type: Option<String>,
    /// Sắp xếp đa trường: `field:dir,field:dir` (vd `sku:asc,created_at:desc`).
    pub sort: Option<String>,
    pub page: Option<u32>,
    pub page_size: Option<u32>,
}
