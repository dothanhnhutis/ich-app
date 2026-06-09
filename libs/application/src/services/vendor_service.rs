use std::str::FromStr;

use uuid::Uuid;
use validator::Validate;

use crate::dto::pagination::Paginated;
use crate::dto::vendor_dto::{
    CreateVendorRequest, ListVendorsQuery, UpdateVendorRequest, VendorResponse,
};
use crate::errors::AppError;
use domain::entities::{
    NewVendor, SortDir, VendorFilter, VendorSort, VendorSortField, VendorType, VendorUpdate,
};
use domain::repositories::VendorRepository;

const DEFAULT_PAGE_SIZE: u32 = 20;
const MAX_PAGE_SIZE: u32 = 100;

/// Chuẩn hoá chuỗi: trim, rỗng → None (không lọc / không ghi).
fn norm(s: Option<String>) -> Option<String> {
    s.map(|v| v.trim().to_string()).filter(|v| !v.is_empty())
}

/// Parse loại nhà cung cấp → VendorType, lỗi → 400.
fn parse_vendor_type(s: &str) -> Result<VendorType, AppError> {
    VendorType::from_str(s.trim())
        .map_err(|_| AppError::Validation("Loại nhà cung cấp không hợp lệ".into()))
}

/// Parse chuỗi sort `field:dir,field:dir` → Vec<VendorSort> (thiếu hướng → asc); lỗi → 400.
fn parse_sort(raw: &str) -> Result<Vec<VendorSort>, AppError> {
    let mut out = Vec::new();
    for token in raw.split(',') {
        let token = token.trim();
        if token.is_empty() {
            continue;
        }
        let (f, d) = match token.split_once(':') {
            Some((f, d)) => (f.trim(), d.trim()),
            None => (token, "asc"),
        };
        let field = VendorSortField::from_str(&f.to_lowercase())
            .map_err(|_| AppError::Validation(format!("Trường sắp xếp không hợp lệ: {f}")))?;
        let dir = SortDir::from_str(&d.to_lowercase())
            .map_err(|_| AppError::Validation(format!("Hướng sắp xếp không hợp lệ: {d}")))?;
        out.push(VendorSort { field, dir });
    }
    Ok(out)
}

pub struct VendorService<VR>
where
    VR: VendorRepository,
{
    vendor_repo: VR,
}

impl<VR> VendorService<VR>
where
    VR: VendorRepository,
{
    pub fn new(vendor_repo: VR) -> Self {
        Self { vendor_repo }
    }

    /// Tạo nhà cung cấp mới (cần VENDOR_CREATE — kiểm ở middleware).
    pub async fn create_vendor(
        &self,
        req: CreateVendorRequest,
    ) -> Result<VendorResponse, AppError> {
        req.validate()
            .map_err(|e| AppError::Validation(e.to_string()))?;
        let vendor_type = parse_vendor_type(&req.vendor_type)?;

        let vendor = self
            .vendor_repo
            .create(NewVendor {
                code: req.code.trim().to_string(),
                name: req.name.trim().to_string(),
                vendor_type,
                tax_code: norm(req.tax_code),
                address: norm(req.address),
                phone: norm(req.phone),
                email: norm(req.email),
                notes: norm(req.notes),
            })
            .await?;
        Ok(VendorResponse::from(vendor))
    }

    /// Danh sách nhà cung cấp (lọc + phân trang + sắp xếp) (cần VENDOR_VIEW).
    pub async fn list_vendors(
        &self,
        q: ListVendorsQuery,
    ) -> Result<Paginated<VendorResponse>, AppError> {
        let page = q.page.unwrap_or(1).max(1);
        let page_size = q.page_size.unwrap_or(DEFAULT_PAGE_SIZE).clamp(1, MAX_PAGE_SIZE);

        let sort = match q.sort.as_deref() {
            Some(s) => parse_sort(s)?,
            None => Vec::new(),
        };

        let filter = VendorFilter {
            code: norm(q.code),
            name: norm(q.name),
            vendor_type: norm(q.vendor_type),
            sort,
            limit: page_size as i64,
            offset: ((page - 1) * page_size) as i64,
        };

        let (vendors, total) = self.vendor_repo.list(filter).await?;

        let total_pages = if total == 0 {
            0
        } else {
            ((total as u64).div_ceil(page_size as u64)) as u32
        };

        Ok(Paginated {
            items: vendors.into_iter().map(VendorResponse::from).collect(),
            page,
            page_size,
            total_items: total,
            total_pages,
            has_next: page < total_pages,
            has_prev: page > 1,
        })
    }

    /// Chi tiết một nhà cung cấp (cần VENDOR_VIEW).
    pub async fn get_vendor(&self, id: Uuid) -> Result<VendorResponse, AppError> {
        let vendor = self
            .vendor_repo
            .find_by_id(id)
            .await?
            .ok_or_else(|| AppError::NotFound("Nhà cung cấp không tồn tại".into()))?;
        Ok(VendorResponse::from(vendor))
    }

    /// Cập nhật nhà cung cấp (cần VENDOR_UPDATE).
    pub async fn update_vendor(
        &self,
        id: Uuid,
        req: UpdateVendorRequest,
    ) -> Result<VendorResponse, AppError> {
        req.validate()
            .map_err(|e| AppError::Validation(e.to_string()))?;

        let vendor_type = match req.vendor_type.as_deref() {
            Some(s) => Some(parse_vendor_type(s)?),
            None => None,
        };

        let changes = VendorUpdate {
            code: req.code.map(|c| c.trim().to_string()),
            name: req.name.map(|n| n.trim().to_string()),
            vendor_type,
            tax_code: norm(req.tax_code),
            address: norm(req.address),
            phone: norm(req.phone),
            email: norm(req.email),
            notes: norm(req.notes),
        };

        let updated = self
            .vendor_repo
            .update(id, changes)
            .await?
            .ok_or_else(|| AppError::NotFound("Nhà cung cấp không tồn tại".into()))?;
        Ok(VendorResponse::from(updated))
    }

    /// Xoá mềm nhà cung cấp (cần VENDOR_DELETE).
    pub async fn delete_vendor(&self, id: Uuid) -> Result<(), AppError> {
        self.vendor_repo.soft_delete(id).await?;
        Ok(())
    }
}
