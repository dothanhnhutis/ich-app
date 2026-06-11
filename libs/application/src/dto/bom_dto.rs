use chrono::{DateTime, Utc};
use domain::entities::{Bom, BomLine};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

#[derive(Debug, Serialize)]
pub struct BomResponse {
    pub id: String,
    pub output_item_id: String,
    pub bom_type: String,
    pub code: String,
    pub name: String,
    pub version_no: i32,
    pub status: String,
    pub is_default: bool,
    pub qty_basis: String,
    pub output_qty: f64,
    pub output_uom: String,
    pub effective_from: Option<DateTime<Utc>>,
    pub effective_to: Option<DateTime<Utc>>,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<Bom> for BomResponse {
    fn from(b: Bom) -> Self {
        Self {
            id: b.id.to_string(),
            output_item_id: b.output_item_id.to_string(),
            bom_type: b.bom_type.as_str().to_string(),
            code: b.code,
            name: b.name,
            version_no: b.version_no,
            status: b.status.as_str().to_string(),
            is_default: b.is_default,
            qty_basis: b.qty_basis.as_str().to_string(),
            output_qty: b.output_qty,
            output_uom: b.output_uom,
            effective_from: b.effective_from,
            effective_to: b.effective_to,
            notes: b.notes,
            created_at: b.created_at,
            updated_at: b.updated_at,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct BomLineResponse {
    pub id: String,
    pub bom_id: String,
    pub component_item_id: String,
    pub line_no: i32,
    pub line_type: String,
    pub quantity: f64,
    pub input_uom: Option<String>,
    pub input_qty: Option<f64>,
    pub scrap_pct: f64,
    pub is_gift: bool,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<BomLine> for BomLineResponse {
    fn from(l: BomLine) -> Self {
        Self {
            id: l.id.to_string(),
            bom_id: l.bom_id.to_string(),
            component_item_id: l.component_item_id.to_string(),
            line_no: l.line_no,
            line_type: l.line_type.as_str().to_string(),
            quantity: l.quantity,
            input_uom: l.input_uom,
            input_qty: l.input_qty,
            scrap_pct: l.scrap_pct,
            is_gift: l.is_gift,
            notes: l.notes,
            created_at: l.created_at,
            updated_at: l.updated_at,
        }
    }
}

/// BOM kèm dòng (nested read / kết quả create).
#[derive(Debug, Serialize)]
pub struct BomWithLinesResponse {
    pub bom: BomResponse,
    pub lines: Vec<BomLineResponse>,
}

impl From<(Bom, Vec<BomLine>)> for BomWithLinesResponse {
    fn from((bom, lines): (Bom, Vec<BomLine>)) -> Self {
        Self {
            bom: BomResponse::from(bom),
            lines: lines.into_iter().map(BomLineResponse::from).collect(),
        }
    }
}

/// Một dòng trong payload tạo BOM. `line_type`/enum parse ở service.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CreateBomLineInput {
    pub component_item_id: Uuid,
    pub line_no: Option<i32>,
    pub line_type: Option<String>,
    pub quantity: f64,
    pub input_uom: Option<String>,
    pub input_qty: Option<f64>,
    pub scrap_pct: Option<f64>,
    #[serde(default)]
    pub is_gift: bool,
    pub notes: Option<String>,
}

/// Tạo BOM (header + dòng, transaction). `bom_type`/`status`/`qty_basis` parse ở service.
#[derive(Debug, Deserialize, Validate)]
#[serde(deny_unknown_fields)]
pub struct CreateBomRequest {
    pub output_item_id: Uuid,
    pub bom_type: String,
    #[validate(length(min = 1, max = 50, message = "Mã BOM 1-50 ký tự"))]
    pub code: String,
    #[validate(length(min = 1, max = 255, message = "Tên BOM 1-255 ký tự"))]
    pub name: String,
    pub version_no: Option<i32>,
    pub status: Option<String>,
    #[serde(default)]
    pub is_default: bool,
    pub qty_basis: Option<String>,
    pub output_qty: f64,
    #[validate(length(min = 1, max = 20, message = "Đơn vị đầu ra 1-20 ký tự"))]
    pub output_uom: String,
    pub effective_from: Option<DateTime<Utc>>,
    pub effective_to: Option<DateTime<Utc>>,
    pub notes: Option<String>,
    #[serde(default)]
    pub lines: Vec<CreateBomLineInput>,
}

/// Cập nhật BOM header (tất cả tùy chọn). `output_item_id` & `bom_type` **bị khoá**.
#[derive(Debug, Deserialize, Validate)]
#[serde(deny_unknown_fields)]
pub struct UpdateBomRequest {
    #[validate(length(min = 1, max = 50, message = "Mã BOM 1-50 ký tự"))]
    pub code: Option<String>,
    #[validate(length(min = 1, max = 255, message = "Tên BOM 1-255 ký tự"))]
    pub name: Option<String>,
    pub version_no: Option<i32>,
    pub status: Option<String>,
    pub is_default: Option<bool>,
    pub qty_basis: Option<String>,
    pub output_qty: Option<f64>,
    #[validate(length(min = 1, max = 20, message = "Đơn vị đầu ra 1-20 ký tự"))]
    pub output_uom: Option<String>,
    pub effective_from: Option<DateTime<Utc>>,
    pub effective_to: Option<DateTime<Utc>>,
    pub notes: Option<String>,
}

/// Thêm một dòng vào BOM hiện có.
#[derive(Debug, Deserialize, Validate)]
#[serde(deny_unknown_fields)]
pub struct AddBomLineRequest {
    pub component_item_id: Uuid,
    pub line_no: Option<i32>,
    pub line_type: Option<String>,
    pub quantity: f64,
    pub input_uom: Option<String>,
    pub input_qty: Option<f64>,
    pub scrap_pct: Option<f64>,
    #[serde(default)]
    pub is_gift: bool,
    pub notes: Option<String>,
}

/// Cập nhật một dòng BOM. `component_item_id` **bị khoá** (đổi = xoá + thêm).
#[derive(Debug, Deserialize, Validate)]
#[serde(deny_unknown_fields)]
pub struct UpdateBomLineRequest {
    pub line_no: Option<i32>,
    pub line_type: Option<String>,
    pub quantity: Option<f64>,
    pub input_uom: Option<String>,
    pub input_qty: Option<f64>,
    pub scrap_pct: Option<f64>,
    pub is_gift: Option<bool>,
    pub notes: Option<String>,
}

/// Tham số lọc + phân trang + sắp xếp cho GET /boms.
#[derive(Debug, Deserialize)]
pub struct ListBomsQuery {
    pub output_item_id: Option<Uuid>,
    pub bom_type: Option<String>,
    pub status: Option<String>,
    pub code: Option<String>,
    /// Sắp xếp đa trường: `field:dir,field:dir` (vd `code:asc,version_no:desc`).
    pub sort: Option<String>,
    pub page: Option<u32>,
    pub page_size: Option<u32>,
}
