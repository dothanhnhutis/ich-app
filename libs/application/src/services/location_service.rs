use std::str::FromStr;

use uuid::Uuid;
use validator::Validate;

use crate::dto::location_dto::{
    CreateLocationRequest, ListLocationsQuery, LocationResponse, UpdateLocationRequest,
};
use crate::dto::pagination::Paginated;
use crate::errors::AppError;
use domain::entities::{
    LocationFilter, LocationSort, LocationSortField, LocationUpdate, NewLocation, SortDir,
};
use domain::repositories::LocationRepository;

const DEFAULT_PAGE_SIZE: u32 = 20;
const MAX_PAGE_SIZE: u32 = 100;

/// Chuẩn hoá chuỗi: trim, rỗng → None (không lọc / không ghi).
fn norm(s: Option<String>) -> Option<String> {
    s.map(|v| v.trim().to_string()).filter(|v| !v.is_empty())
}

/// Parse chuỗi sort `field:dir,field:dir` → Vec<LocationSort> (thiếu hướng → asc); lỗi → 400.
fn parse_sort(raw: &str) -> Result<Vec<LocationSort>, AppError> {
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
        let field = LocationSortField::from_str(&f.to_lowercase())
            .map_err(|_| AppError::Validation(format!("Trường sắp xếp không hợp lệ: {f}")))?;
        let dir = SortDir::from_str(&d.to_lowercase())
            .map_err(|_| AppError::Validation(format!("Hướng sắp xếp không hợp lệ: {d}")))?;
        out.push(LocationSort { field, dir });
    }
    Ok(out)
}

pub struct LocationService<LR>
where
    LR: LocationRepository,
{
    location_repo: LR,
}

impl<LR> LocationService<LR>
where
    LR: LocationRepository,
{
    pub fn new(location_repo: LR) -> Self {
        Self { location_repo }
    }

    /// Tạo kho mới (cần LOCATION_CREATE — kiểm ở middleware).
    pub async fn create_location(
        &self,
        req: CreateLocationRequest,
    ) -> Result<LocationResponse, AppError> {
        req.validate()
            .map_err(|e| AppError::Validation(e.to_string()))?;

        let location = self
            .location_repo
            .create(NewLocation {
                code: req.code.trim().to_string(),
                name: req.name.trim().to_string(),
                address: norm(req.address),
            })
            .await?;
        Ok(LocationResponse::from(location))
    }

    /// Danh sách kho (lọc + phân trang + sắp xếp) (cần LOCATION_VIEW).
    pub async fn list_locations(
        &self,
        q: ListLocationsQuery,
    ) -> Result<Paginated<LocationResponse>, AppError> {
        let page = q.page.unwrap_or(1).max(1);
        let page_size = q.page_size.unwrap_or(DEFAULT_PAGE_SIZE).clamp(1, MAX_PAGE_SIZE);

        let sort = match q.sort.as_deref() {
            Some(s) => parse_sort(s)?,
            None => Vec::new(),
        };

        let filter = LocationFilter {
            code: norm(q.code),
            name: norm(q.name),
            sort,
            limit: page_size as i64,
            offset: ((page - 1) * page_size) as i64,
        };

        let (locations, total) = self.location_repo.list(filter).await?;

        let total_pages = if total == 0 {
            0
        } else {
            ((total as u64).div_ceil(page_size as u64)) as u32
        };

        Ok(Paginated {
            items: locations.into_iter().map(LocationResponse::from).collect(),
            page,
            page_size,
            total_items: total,
            total_pages,
            has_next: page < total_pages,
            has_prev: page > 1,
        })
    }

    /// Chi tiết một kho (cần LOCATION_VIEW). 404 nếu không tồn tại / đã xoá mềm.
    pub async fn get_location(&self, id: Uuid) -> Result<LocationResponse, AppError> {
        let location = self
            .location_repo
            .find_by_id(id)
            .await?
            .ok_or_else(|| AppError::NotFound("Kho không tồn tại".into()))?;
        Ok(LocationResponse::from(location))
    }

    /// Cập nhật kho (code/name/address) (cần LOCATION_UPDATE). 404 nếu không tồn tại / đã xoá.
    pub async fn update_location(
        &self,
        id: Uuid,
        req: UpdateLocationRequest,
    ) -> Result<LocationResponse, AppError> {
        req.validate()
            .map_err(|e| AppError::Validation(e.to_string()))?;

        let changes = LocationUpdate {
            code: req.code.map(|c| c.trim().to_string()),
            name: req.name.map(|n| n.trim().to_string()),
            address: norm(req.address),
        };

        let updated = self
            .location_repo
            .update(id, changes)
            .await?
            .ok_or_else(|| AppError::NotFound("Kho không tồn tại".into()))?;
        Ok(LocationResponse::from(updated))
    }

    /// Xoá mềm kho (cần LOCATION_DELETE). 404 nếu không tồn tại / đã xoá;
    /// chặn nếu còn khu vực con đang hoạt động.
    pub async fn delete_location(&self, id: Uuid) -> Result<(), AppError> {
        self.location_repo
            .find_by_id(id)
            .await?
            .ok_or_else(|| AppError::NotFound("Kho không tồn tại".into()))?;
        if self.location_repo.has_active_zones(id).await? {
            return Err(AppError::Validation(
                "Không thể xoá kho: vẫn còn khu vực đang hoạt động".into(),
            ));
        }
        self.location_repo.soft_delete(id).await?;
        Ok(())
    }
}
