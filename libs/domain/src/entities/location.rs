use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::entities::SortDir;

/// Kho vật lý (toà nhà, chi nhánh). `deleted_at` chỉ lọc trong SQL — không có ở entity.
#[derive(Debug, Clone)]
pub struct Location {
    pub id: Uuid,
    pub code: String,
    pub name: String,
    pub address: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Dữ liệu để tạo kho mới.
#[derive(Debug, Clone)]
pub struct NewLocation {
    pub code: String,
    pub name: String,
    pub address: Option<String>,
}

/// Thay đổi cho cập nhật kho — chỉ field `Some` mới được ghi (COALESCE).
#[derive(Debug, Clone, Default)]
pub struct LocationUpdate {
    pub code: Option<String>,
    pub name: Option<String>,
    pub address: Option<String>,
}

/// Trường được phép sắp xếp cho danh sách kho (whitelist).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LocationSortField {
    Code,
    Name,
    CreatedAt,
    UpdatedAt,
}

impl std::str::FromStr for LocationSortField {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "code" => Ok(LocationSortField::Code),
            "name" => Ok(LocationSortField::Name),
            "created_at" => Ok(LocationSortField::CreatedAt),
            "updated_at" => Ok(LocationSortField::UpdatedAt),
            other => Err(format!("Unknown LocationSortField: {}", other)),
        }
    }
}

/// Một tiêu chí sắp xếp (trường + hướng).
#[derive(Debug, Clone, Copy)]
pub struct LocationSort {
    pub field: LocationSortField,
    pub dir: SortDir,
}

/// Điều kiện lọc + phân trang + sắp xếp cho danh sách kho.
#[derive(Debug, Clone)]
pub struct LocationFilter {
    pub code: Option<String>,
    pub name: Option<String>,
    /// Thứ tự sắp xếp (ưu tiên theo vị trí); rỗng = mặc định created_at DESC.
    pub sort: Vec<LocationSort>,
    pub limit: i64,
    pub offset: i64,
}
