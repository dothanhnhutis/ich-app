use chrono::{DateTime, Utc};
use domain::entities::Vendor;
use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Debug, Serialize)]
pub struct VendorResponse {
    pub id: String,
    pub code: String,
    pub name: String,
    pub vendor_type: String,
    pub tax_code: Option<String>,
    pub address: Option<String>,
    pub phone: Option<String>,
    pub email: Option<String>,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<Vendor> for VendorResponse {
    fn from(v: Vendor) -> Self {
        Self {
            id: v.id.to_string(),
            code: v.code,
            name: v.name,
            vendor_type: v.vendor_type.as_str().to_string(),
            tax_code: v.tax_code,
            address: v.address,
            phone: v.phone,
            email: v.email,
            notes: v.notes,
            created_at: v.created_at,
            updated_at: v.updated_at,
        }
    }
}

/// Tạo nhà cung cấp mới. `vendor_type` parse + kiểm ở service.
#[derive(Debug, Deserialize, Validate)]
#[serde(deny_unknown_fields)]
pub struct CreateVendorRequest {
    #[validate(length(min = 1, max = 50, message = "Mã nhà cung cấp 1-50 ký tự"))]
    pub code: String,
    #[validate(length(min = 1, max = 255, message = "Tên nhà cung cấp 1-255 ký tự"))]
    pub name: String,
    pub vendor_type: String,
    #[validate(length(max = 50, message = "Mã số thuế tối đa 50 ký tự"))]
    pub tax_code: Option<String>,
    #[validate(length(max = 255, message = "Địa chỉ tối đa 255 ký tự"))]
    pub address: Option<String>,
    #[validate(length(max = 50, message = "Số điện thoại tối đa 50 ký tự"))]
    pub phone: Option<String>,
    #[validate(email(message = "Email không hợp lệ"))]
    pub email: Option<String>,
    pub notes: Option<String>,
}

/// Cập nhật nhà cung cấp (tất cả tùy chọn).
#[derive(Debug, Deserialize, Validate)]
#[serde(deny_unknown_fields)]
pub struct UpdateVendorRequest {
    #[validate(length(min = 1, max = 50, message = "Mã nhà cung cấp 1-50 ký tự"))]
    pub code: Option<String>,
    #[validate(length(min = 1, max = 255, message = "Tên nhà cung cấp 1-255 ký tự"))]
    pub name: Option<String>,
    pub vendor_type: Option<String>,
    #[validate(length(max = 50, message = "Mã số thuế tối đa 50 ký tự"))]
    pub tax_code: Option<String>,
    #[validate(length(max = 255, message = "Địa chỉ tối đa 255 ký tự"))]
    pub address: Option<String>,
    #[validate(length(max = 50, message = "Số điện thoại tối đa 50 ký tự"))]
    pub phone: Option<String>,
    #[validate(email(message = "Email không hợp lệ"))]
    pub email: Option<String>,
    pub notes: Option<String>,
}

/// Tham số lọc + phân trang + sắp xếp cho GET /vendors.
#[derive(Debug, Deserialize)]
pub struct ListVendorsQuery {
    pub code: Option<String>,
    pub name: Option<String>,
    pub vendor_type: Option<String>,
    /// Sắp xếp đa trường: `field:dir,field:dir` (vd `name:asc,created_at:desc`).
    pub sort: Option<String>,
    pub page: Option<u32>,
    pub page_size: Option<u32>,
}
