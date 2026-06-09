use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::entities::SortDir;

/// Loại nhà cung cấp (khớp CHECK chk_vendors_type).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VendorType {
    Supplier,
    Manufacturer,
    Both,
}

impl VendorType {
    pub fn as_str(&self) -> &'static str {
        match self {
            VendorType::Supplier => "SUPPLIER",
            VendorType::Manufacturer => "MANUFACTURER",
            VendorType::Both => "BOTH",
        }
    }
}

impl std::str::FromStr for VendorType {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "SUPPLIER" => Ok(VendorType::Supplier),
            "MANUFACTURER" => Ok(VendorType::Manufacturer),
            "BOTH" => Ok(VendorType::Both),
            other => Err(format!("Unknown VendorType: {}", other)),
        }
    }
}

/// Nhà cung cấp. `deleted_at` chỉ lọc trong SQL — không có ở entity.
#[derive(Debug, Clone)]
pub struct Vendor {
    pub id: Uuid,
    pub code: String,
    pub name: String,
    pub vendor_type: VendorType,
    pub tax_code: Option<String>,
    pub address: Option<String>,
    pub phone: Option<String>,
    pub email: Option<String>,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Dữ liệu để tạo nhà cung cấp mới.
#[derive(Debug, Clone)]
pub struct NewVendor {
    pub code: String,
    pub name: String,
    pub vendor_type: VendorType,
    pub tax_code: Option<String>,
    pub address: Option<String>,
    pub phone: Option<String>,
    pub email: Option<String>,
    pub notes: Option<String>,
}

/// Thay đổi cho cập nhật nhà cung cấp — chỉ field `Some` mới được ghi (COALESCE).
#[derive(Debug, Clone, Default)]
pub struct VendorUpdate {
    pub code: Option<String>,
    pub name: Option<String>,
    pub vendor_type: Option<VendorType>,
    pub tax_code: Option<String>,
    pub address: Option<String>,
    pub phone: Option<String>,
    pub email: Option<String>,
    pub notes: Option<String>,
}

/// Trường được phép sắp xếp cho danh sách nhà cung cấp (whitelist).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VendorSortField {
    Code,
    Name,
    VendorType,
    CreatedAt,
    UpdatedAt,
}

impl std::str::FromStr for VendorSortField {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "code" => Ok(VendorSortField::Code),
            "name" => Ok(VendorSortField::Name),
            "vendor_type" => Ok(VendorSortField::VendorType),
            "created_at" => Ok(VendorSortField::CreatedAt),
            "updated_at" => Ok(VendorSortField::UpdatedAt),
            other => Err(format!("Unknown VendorSortField: {}", other)),
        }
    }
}

/// Một tiêu chí sắp xếp (trường + hướng).
#[derive(Debug, Clone, Copy)]
pub struct VendorSort {
    pub field: VendorSortField,
    pub dir: SortDir,
}

/// Điều kiện lọc + phân trang + sắp xếp cho danh sách nhà cung cấp.
#[derive(Debug, Clone)]
pub struct VendorFilter {
    pub code: Option<String>,
    pub name: Option<String>,
    pub vendor_type: Option<String>,
    pub sort: Vec<VendorSort>,
    pub limit: i64,
    pub offset: i64,
}
