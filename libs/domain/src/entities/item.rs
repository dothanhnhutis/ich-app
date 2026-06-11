use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::entities::SortDir;

/// Loại vật tư (khớp CHECK chk_items_type). Master dùng chung cho mọi loại.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ItemType {
    RawMaterial,
    Packaging,
    Utility,
    SemiFinished,
    FinishedGoods,
}

impl ItemType {
    pub fn as_str(&self) -> &'static str {
        match self {
            ItemType::RawMaterial => "RAW_MATERIAL",
            ItemType::Packaging => "PACKAGING",
            ItemType::Utility => "UTILITY",
            ItemType::SemiFinished => "SEMI_FINISHED",
            ItemType::FinishedGoods => "FINISHED_GOODS",
        }
    }

    /// Item có thể có BOM (tự sản xuất): bán thành phẩm / thành phẩm.
    pub fn can_have_bom(&self) -> bool {
        matches!(self, ItemType::SemiFinished | ItemType::FinishedGoods)
    }
}

impl std::str::FromStr for ItemType {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "RAW_MATERIAL" => Ok(ItemType::RawMaterial),
            "PACKAGING" => Ok(ItemType::Packaging),
            "UTILITY" => Ok(ItemType::Utility),
            "SEMI_FINISHED" => Ok(ItemType::SemiFinished),
            "FINISHED_GOODS" => Ok(ItemType::FinishedGoods),
            other => Err(format!("Unknown ItemType: {}", other)),
        }
    }
}

/// Cấp bao bì (chỉ khi type=PACKAGING — khớp CHECK chk_items_pkg_level).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PackagingLevel {
    Primary,
    Secondary,
    Tertiary,
    Carton,
}

impl PackagingLevel {
    pub fn as_str(&self) -> &'static str {
        match self {
            PackagingLevel::Primary => "PRIMARY",
            PackagingLevel::Secondary => "SECONDARY",
            PackagingLevel::Tertiary => "TERTIARY",
            PackagingLevel::Carton => "CARTON",
        }
    }
}

impl std::str::FromStr for PackagingLevel {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "PRIMARY" => Ok(PackagingLevel::Primary),
            "SECONDARY" => Ok(PackagingLevel::Secondary),
            "TERTIARY" => Ok(PackagingLevel::Tertiary),
            "CARTON" => Ok(PackagingLevel::Carton),
            other => Err(format!("Unknown PackagingLevel: {}", other)),
        }
    }
}

/// Vật tư (item master). `deleted_at` chỉ lọc trong SQL — không có ở entity.
#[derive(Debug, Clone)]
pub struct Item {
    pub id: Uuid,
    pub sku: String,
    pub name: String,
    pub item_type: ItemType,
    pub base_uom: String,
    pub packaging_level: Option<PackagingLevel>,
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

/// Dữ liệu để tạo vật tư mới.
#[derive(Debug, Clone)]
pub struct NewItem {
    pub sku: String,
    pub name: String,
    pub item_type: ItemType,
    pub base_uom: String,
    pub packaging_level: Option<PackagingLevel>,
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
}

/// Thay đổi cho cập nhật vật tư — chỉ field `Some` mới được ghi (COALESCE).
/// `item_type` và `base_uom` **bị khoá** (immutable v1) → không có ở đây.
#[derive(Debug, Clone, Default)]
pub struct ItemUpdate {
    pub sku: Option<String>,
    pub name: Option<String>,
    pub packaging_level: Option<PackagingLevel>,
    pub is_purchasable: Option<bool>,
    pub is_sellable: Option<bool>,
    pub has_bom: Option<bool>,
    pub is_lot_controlled: Option<bool>,
    pub is_phantom: Option<bool>,
    pub density_g_ml: Option<f64>,
    pub shelf_life_days: Option<i32>,
    pub pao_months: Option<i16>,
    pub inci_name: Option<String>,
    pub cas_number: Option<String>,
    pub description: Option<String>,
}

/// Trường được phép sắp xếp cho danh sách vật tư (whitelist).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ItemSortField {
    Sku,
    Name,
    ItemType,
    CreatedAt,
    UpdatedAt,
}

impl std::str::FromStr for ItemSortField {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "sku" => Ok(ItemSortField::Sku),
            "name" => Ok(ItemSortField::Name),
            "type" => Ok(ItemSortField::ItemType),
            "created_at" => Ok(ItemSortField::CreatedAt),
            "updated_at" => Ok(ItemSortField::UpdatedAt),
            other => Err(format!("Unknown ItemSortField: {}", other)),
        }
    }
}

/// Một tiêu chí sắp xếp (trường + hướng).
#[derive(Debug, Clone, Copy)]
pub struct ItemSort {
    pub field: ItemSortField,
    pub dir: SortDir,
}

/// Điều kiện lọc + phân trang + sắp xếp cho danh sách vật tư.
/// `allowed_types`: giới hạn theo quyền per-type (None = không giới hạn).
#[derive(Debug, Clone)]
pub struct ItemFilter {
    pub sku: Option<String>,
    pub name: Option<String>,
    pub item_type: Option<String>,
    pub allowed_types: Option<Vec<String>>,
    pub sort: Vec<ItemSort>,
    pub limit: i64,
    pub offset: i64,
}
