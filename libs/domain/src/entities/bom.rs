use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::entities::SortDir;

/// Loại BOM (khớp CHECK chk_boms_type).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BomType {
    Formula,
    Packing,
}

impl BomType {
    pub fn as_str(&self) -> &'static str {
        match self {
            BomType::Formula => "FORMULA",
            BomType::Packing => "PACKING",
        }
    }
}

impl std::str::FromStr for BomType {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "FORMULA" => Ok(BomType::Formula),
            "PACKING" => Ok(BomType::Packing),
            other => Err(format!("Unknown BomType: {}", other)),
        }
    }
}

/// Trạng thái BOM (khớp CHECK chk_boms_status).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BomStatus {
    Draft,
    Active,
    Obsolete,
}

impl BomStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            BomStatus::Draft => "DRAFT",
            BomStatus::Active => "ACTIVE",
            BomStatus::Obsolete => "OBSOLETE",
        }
    }
}

impl std::str::FromStr for BomStatus {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "DRAFT" => Ok(BomStatus::Draft),
            "ACTIVE" => Ok(BomStatus::Active),
            "OBSOLETE" => Ok(BomStatus::Obsolete),
            other => Err(format!("Unknown BomStatus: {}", other)),
        }
    }
}

/// Cơ sở định lượng (khớp CHECK chk_boms_qty_basis).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QtyBasis {
    Percent,
    Absolute,
}

impl QtyBasis {
    pub fn as_str(&self) -> &'static str {
        match self {
            QtyBasis::Percent => "PERCENT",
            QtyBasis::Absolute => "ABSOLUTE",
        }
    }
}

impl std::str::FromStr for QtyBasis {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "PERCENT" => Ok(QtyBasis::Percent),
            "ABSOLUTE" => Ok(QtyBasis::Absolute),
            other => Err(format!("Unknown QtyBasis: {}", other)),
        }
    }
}

/// Loại dòng BOM (khớp CHECK chk_bom_lines_type).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BomLineType {
    Item,
    Phantom,
}

impl BomLineType {
    pub fn as_str(&self) -> &'static str {
        match self {
            BomLineType::Item => "ITEM",
            BomLineType::Phantom => "PHANTOM",
        }
    }
}

impl std::str::FromStr for BomLineType {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "ITEM" => Ok(BomLineType::Item),
            "PHANTOM" => Ok(BomLineType::Phantom),
            other => Err(format!("Unknown BomLineType: {}", other)),
        }
    }
}

/// Header BOM. `deleted_at` chỉ lọc trong SQL — không có ở entity.
#[derive(Debug, Clone)]
pub struct Bom {
    pub id: Uuid,
    pub output_item_id: Uuid,
    pub bom_type: BomType,
    pub code: String,
    pub name: String,
    pub version_no: i32,
    pub status: BomStatus,
    pub is_default: bool,
    pub qty_basis: QtyBasis,
    pub output_qty: f64,
    pub output_uom: String,
    pub effective_from: Option<DateTime<Utc>>,
    pub effective_to: Option<DateTime<Utc>>,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Dữ liệu tạo BOM header mới.
#[derive(Debug, Clone)]
pub struct NewBom {
    pub output_item_id: Uuid,
    pub bom_type: BomType,
    pub code: String,
    pub name: String,
    pub version_no: i32,
    pub status: BomStatus,
    pub is_default: bool,
    pub qty_basis: QtyBasis,
    pub output_qty: f64,
    pub output_uom: String,
    pub effective_from: Option<DateTime<Utc>>,
    pub effective_to: Option<DateTime<Utc>>,
    pub notes: Option<String>,
}

/// Cập nhật BOM header — chỉ field `Some` được ghi (COALESCE).
/// `output_item_id` & `bom_type` **bị khoá** (immutable v1) → không có ở đây.
#[derive(Debug, Clone, Default)]
pub struct BomUpdate {
    pub code: Option<String>,
    pub name: Option<String>,
    pub version_no: Option<i32>,
    pub status: Option<BomStatus>,
    pub is_default: Option<bool>,
    pub qty_basis: Option<QtyBasis>,
    pub output_qty: Option<f64>,
    pub output_uom: Option<String>,
    pub effective_from: Option<DateTime<Utc>>,
    pub effective_to: Option<DateTime<Utc>>,
    pub notes: Option<String>,
}

/// Một dòng nguyên liệu trong BOM.
#[derive(Debug, Clone)]
pub struct BomLine {
    pub id: Uuid,
    pub bom_id: Uuid,
    pub component_item_id: Uuid,
    pub line_no: i32,
    pub line_type: BomLineType,
    pub quantity: f64,
    pub input_uom: Option<String>,
    pub input_qty: Option<f64>,
    pub scrap_pct: f64,
    pub is_gift: bool,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Dữ liệu tạo dòng BOM mới (`bom_id` do repo gán khi tạo/thêm).
#[derive(Debug, Clone)]
pub struct NewBomLine {
    pub component_item_id: Uuid,
    pub line_no: i32,
    pub line_type: BomLineType,
    pub quantity: f64,
    pub input_uom: Option<String>,
    pub input_qty: Option<f64>,
    pub scrap_pct: f64,
    pub is_gift: bool,
    pub notes: Option<String>,
}

/// Cập nhật dòng BOM. `component_item_id` **bị khoá** (đổi component = xoá + thêm dòng).
#[derive(Debug, Clone, Default)]
pub struct BomLineUpdate {
    pub line_no: Option<i32>,
    pub line_type: Option<BomLineType>,
    pub quantity: Option<f64>,
    pub input_uom: Option<String>,
    pub input_qty: Option<f64>,
    pub scrap_pct: Option<f64>,
    pub is_gift: Option<bool>,
    pub notes: Option<String>,
}

/// Trường được phép sắp xếp cho danh sách BOM (whitelist).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BomSortField {
    Code,
    Name,
    Status,
    VersionNo,
    CreatedAt,
    UpdatedAt,
}

impl std::str::FromStr for BomSortField {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "code" => Ok(BomSortField::Code),
            "name" => Ok(BomSortField::Name),
            "status" => Ok(BomSortField::Status),
            "version_no" => Ok(BomSortField::VersionNo),
            "created_at" => Ok(BomSortField::CreatedAt),
            "updated_at" => Ok(BomSortField::UpdatedAt),
            other => Err(format!("Unknown BomSortField: {}", other)),
        }
    }
}

/// Một tiêu chí sắp xếp (trường + hướng).
#[derive(Debug, Clone, Copy)]
pub struct BomSort {
    pub field: BomSortField,
    pub dir: SortDir,
}

/// Điều kiện lọc + phân trang + sắp xếp cho danh sách BOM.
#[derive(Debug, Clone)]
pub struct BomFilter {
    pub output_item_id: Option<Uuid>,
    pub bom_type: Option<String>,
    pub status: Option<String>,
    pub code: Option<String>,
    pub sort: Vec<BomSort>,
    pub limit: i64,
    pub offset: i64,
}
