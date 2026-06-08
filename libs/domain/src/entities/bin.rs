use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::entities::SortDir;

/// Ô/kệ lưu trữ trong khu vực. `deleted_at` chỉ lọc trong SQL — không có ở entity.
#[derive(Debug, Clone)]
pub struct Bin {
    pub id: Uuid,
    pub zone_id: Uuid,
    pub code: String,
    pub name: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Dữ liệu để tạo kệ mới.
#[derive(Debug, Clone)]
pub struct NewBin {
    pub zone_id: Uuid,
    pub code: String,
    pub name: String,
}

/// Thay đổi cho cập nhật kệ — chỉ field `Some` mới được ghi (COALESCE).
#[derive(Debug, Clone, Default)]
pub struct BinUpdate {
    pub zone_id: Option<Uuid>,
    pub code: Option<String>,
    pub name: Option<String>,
}

/// Trường được phép sắp xếp cho danh sách kệ (whitelist).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinSortField {
    Code,
    Name,
    CreatedAt,
    UpdatedAt,
}

impl std::str::FromStr for BinSortField {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "code" => Ok(BinSortField::Code),
            "name" => Ok(BinSortField::Name),
            "created_at" => Ok(BinSortField::CreatedAt),
            "updated_at" => Ok(BinSortField::UpdatedAt),
            other => Err(format!("Unknown BinSortField: {}", other)),
        }
    }
}

/// Một tiêu chí sắp xếp (trường + hướng).
#[derive(Debug, Clone, Copy)]
pub struct BinSort {
    pub field: BinSortField,
    pub dir: SortDir,
}

/// Điều kiện lọc + phân trang + sắp xếp cho danh sách kệ.
#[derive(Debug, Clone)]
pub struct BinFilter {
    pub zone_id: Option<Uuid>,
    pub code: Option<String>,
    pub name: Option<String>,
    pub sort: Vec<BinSort>,
    pub limit: i64,
    pub offset: i64,
}
