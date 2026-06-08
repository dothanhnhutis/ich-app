use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::entities::SortDir;

/// Loại khu vực kho (khớp CHECK chk_warehouse_zones_type).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ZoneType {
    FinishedGoods,
    RawMaterial,
    Packaging,
    Quarantine,
    Reject,
    Return,
    Utility,
}

impl ZoneType {
    pub fn as_str(&self) -> &'static str {
        match self {
            ZoneType::FinishedGoods => "FINISHED_GOODS",
            ZoneType::RawMaterial => "RAW_MATERIAL",
            ZoneType::Packaging => "PACKAGING",
            ZoneType::Quarantine => "QUARANTINE",
            ZoneType::Reject => "REJECT",
            ZoneType::Return => "RETURN",
            ZoneType::Utility => "UTILITY",
        }
    }
}

impl std::str::FromStr for ZoneType {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "FINISHED_GOODS" => Ok(ZoneType::FinishedGoods),
            "RAW_MATERIAL" => Ok(ZoneType::RawMaterial),
            "PACKAGING" => Ok(ZoneType::Packaging),
            "QUARANTINE" => Ok(ZoneType::Quarantine),
            "REJECT" => Ok(ZoneType::Reject),
            "RETURN" => Ok(ZoneType::Return),
            "UTILITY" => Ok(ZoneType::Utility),
            other => Err(format!("Unknown ZoneType: {}", other)),
        }
    }
}

/// Khu vực trong kho. `deleted_at` chỉ lọc trong SQL — không có ở entity.
#[derive(Debug, Clone)]
pub struct Zone {
    pub id: Uuid,
    pub location_id: Uuid,
    pub name: String,
    pub zone_type: ZoneType,
    pub temp_min_c: Option<f64>,
    pub temp_max_c: Option<f64>,
    pub humidity_max_pct: Option<f64>,
    pub is_light_protected: bool,
    pub is_ventilated: bool,
    pub is_explosion_proof: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Dữ liệu để tạo khu vực mới.
#[derive(Debug, Clone)]
pub struct NewZone {
    pub location_id: Uuid,
    pub name: String,
    pub zone_type: ZoneType,
    pub temp_min_c: Option<f64>,
    pub temp_max_c: Option<f64>,
    pub humidity_max_pct: Option<f64>,
    pub is_light_protected: bool,
    pub is_ventilated: bool,
    pub is_explosion_proof: bool,
}

/// Thay đổi cho cập nhật khu vực — chỉ field `Some` mới được ghi (COALESCE).
#[derive(Debug, Clone, Default)]
pub struct ZoneUpdate {
    pub location_id: Option<Uuid>,
    pub name: Option<String>,
    pub zone_type: Option<ZoneType>,
    pub temp_min_c: Option<f64>,
    pub temp_max_c: Option<f64>,
    pub humidity_max_pct: Option<f64>,
    pub is_light_protected: Option<bool>,
    pub is_ventilated: Option<bool>,
    pub is_explosion_proof: Option<bool>,
}

/// Trường được phép sắp xếp cho danh sách khu vực (whitelist).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ZoneSortField {
    Name,
    ZoneType,
    CreatedAt,
    UpdatedAt,
}

impl std::str::FromStr for ZoneSortField {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "name" => Ok(ZoneSortField::Name),
            "zone_type" => Ok(ZoneSortField::ZoneType),
            "created_at" => Ok(ZoneSortField::CreatedAt),
            "updated_at" => Ok(ZoneSortField::UpdatedAt),
            other => Err(format!("Unknown ZoneSortField: {}", other)),
        }
    }
}

/// Một tiêu chí sắp xếp (trường + hướng).
#[derive(Debug, Clone, Copy)]
pub struct ZoneSort {
    pub field: ZoneSortField,
    pub dir: SortDir,
}

/// Điều kiện lọc + phân trang + sắp xếp cho danh sách khu vực.
#[derive(Debug, Clone)]
pub struct ZoneFilter {
    pub location_id: Option<Uuid>,
    pub name: Option<String>,
    pub zone_type: Option<String>,
    pub sort: Vec<ZoneSort>,
    pub limit: i64,
    pub offset: i64,
}
