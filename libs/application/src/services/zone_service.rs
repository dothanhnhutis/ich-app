use std::str::FromStr;

use uuid::Uuid;
use validator::Validate;

use crate::dto::pagination::Paginated;
use crate::dto::zone_dto::{CreateZoneRequest, ListZonesQuery, UpdateZoneRequest, ZoneResponse};
use crate::errors::AppError;
use domain::entities::{NewZone, SortDir, ZoneFilter, ZoneSort, ZoneSortField, ZoneType, ZoneUpdate};
use domain::repositories::{LocationRepository, ZoneRepository};

const DEFAULT_PAGE_SIZE: u32 = 20;
const MAX_PAGE_SIZE: u32 = 100;

/// Chuẩn hoá chuỗi: trim, rỗng → None (không lọc).
fn norm(s: Option<String>) -> Option<String> {
    s.map(|v| v.trim().to_string()).filter(|v| !v.is_empty())
}

/// Parse loại khu vực → ZoneType, lỗi → 400.
fn parse_zone_type(s: &str) -> Result<ZoneType, AppError> {
    ZoneType::from_str(s.trim())
        .map_err(|_| AppError::Validation("Loại khu vực không hợp lệ".into()))
}

/// Parse chuỗi sort `field:dir,field:dir` → Vec<ZoneSort> (thiếu hướng → asc); lỗi → 400.
fn parse_sort(raw: &str) -> Result<Vec<ZoneSort>, AppError> {
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
        let field = ZoneSortField::from_str(&f.to_lowercase())
            .map_err(|_| AppError::Validation(format!("Trường sắp xếp không hợp lệ: {f}")))?;
        let dir = SortDir::from_str(&d.to_lowercase())
            .map_err(|_| AppError::Validation(format!("Hướng sắp xếp không hợp lệ: {d}")))?;
        out.push(ZoneSort { field, dir });
    }
    Ok(out)
}

pub struct ZoneService<ZR, LR>
where
    ZR: ZoneRepository,
    LR: LocationRepository,
{
    zone_repo: ZR,
    location_repo: LR,
}

impl<ZR, LR> ZoneService<ZR, LR>
where
    ZR: ZoneRepository,
    LR: LocationRepository,
{
    pub fn new(zone_repo: ZR, location_repo: LR) -> Self {
        Self {
            zone_repo,
            location_repo,
        }
    }

    /// Kho cha phải tồn tại & chưa xoá mềm (FK không bắt được cha đã xoá mềm).
    async fn ensure_location_active(&self, location_id: Uuid) -> Result<(), AppError> {
        self.location_repo
            .find_by_id(location_id)
            .await?
            .ok_or_else(|| AppError::Validation("Kho không tồn tại".into()))?;
        Ok(())
    }

    /// Tạo khu vực mới (cần ZONE_CREATE — kiểm ở middleware).
    pub async fn create_zone(&self, req: CreateZoneRequest) -> Result<ZoneResponse, AppError> {
        req.validate()
            .map_err(|e| AppError::Validation(e.to_string()))?;
        let zone_type = parse_zone_type(&req.zone_type)?;
        self.ensure_location_active(req.location_id).await?;

        let zone = self
            .zone_repo
            .create(NewZone {
                location_id: req.location_id,
                name: req.name.trim().to_string(),
                zone_type,
                temp_min_c: req.temp_min_c,
                temp_max_c: req.temp_max_c,
                humidity_max_pct: req.humidity_max_pct,
                is_light_protected: req.is_light_protected,
                is_ventilated: req.is_ventilated,
                is_explosion_proof: req.is_explosion_proof,
            })
            .await?;
        Ok(ZoneResponse::from(zone))
    }

    /// Danh sách khu vực (lọc + phân trang + sắp xếp) (cần ZONE_VIEW).
    pub async fn list_zones(
        &self,
        q: ListZonesQuery,
    ) -> Result<Paginated<ZoneResponse>, AppError> {
        let page = q.page.unwrap_or(1).max(1);
        let page_size = q.page_size.unwrap_or(DEFAULT_PAGE_SIZE).clamp(1, MAX_PAGE_SIZE);

        let sort = match q.sort.as_deref() {
            Some(s) => parse_sort(s)?,
            None => Vec::new(),
        };

        let filter = ZoneFilter {
            location_id: q.location_id,
            name: norm(q.name),
            zone_type: norm(q.zone_type),
            sort,
            limit: page_size as i64,
            offset: ((page - 1) * page_size) as i64,
        };

        let (zones, total) = self.zone_repo.list(filter).await?;

        let total_pages = if total == 0 {
            0
        } else {
            ((total as u64).div_ceil(page_size as u64)) as u32
        };

        Ok(Paginated {
            items: zones.into_iter().map(ZoneResponse::from).collect(),
            page,
            page_size,
            total_items: total,
            total_pages,
            has_next: page < total_pages,
            has_prev: page > 1,
        })
    }

    /// Chi tiết một khu vực (cần ZONE_VIEW).
    pub async fn get_zone(&self, id: Uuid) -> Result<ZoneResponse, AppError> {
        let zone = self
            .zone_repo
            .find_by_id(id)
            .await?
            .ok_or_else(|| AppError::NotFound("Khu vực không tồn tại".into()))?;
        Ok(ZoneResponse::from(zone))
    }

    /// Cập nhật khu vực (cần ZONE_UPDATE). Cho đổi kho cha (kiểm cha mới còn hoạt động).
    pub async fn update_zone(
        &self,
        id: Uuid,
        req: UpdateZoneRequest,
    ) -> Result<ZoneResponse, AppError> {
        req.validate()
            .map_err(|e| AppError::Validation(e.to_string()))?;

        let zone_type = match req.zone_type.as_deref() {
            Some(s) => Some(parse_zone_type(s)?),
            None => None,
        };
        if let Some(location_id) = req.location_id {
            self.ensure_location_active(location_id).await?;
        }

        let changes = ZoneUpdate {
            location_id: req.location_id,
            name: req.name.map(|n| n.trim().to_string()),
            zone_type,
            temp_min_c: req.temp_min_c,
            temp_max_c: req.temp_max_c,
            humidity_max_pct: req.humidity_max_pct,
            is_light_protected: req.is_light_protected,
            is_ventilated: req.is_ventilated,
            is_explosion_proof: req.is_explosion_proof,
        };

        let updated = self
            .zone_repo
            .update(id, changes)
            .await?
            .ok_or_else(|| AppError::NotFound("Khu vực không tồn tại".into()))?;
        Ok(ZoneResponse::from(updated))
    }

    /// Xoá mềm khu vực (cần ZONE_DELETE). Chặn nếu còn kệ con đang hoạt động.
    pub async fn delete_zone(&self, id: Uuid) -> Result<(), AppError> {
        self.zone_repo
            .find_by_id(id)
            .await?
            .ok_or_else(|| AppError::NotFound("Khu vực không tồn tại".into()))?;
        if self.zone_repo.has_active_bins(id).await? {
            return Err(AppError::Validation(
                "Không thể xoá khu vực: vẫn còn kệ lưu trữ đang hoạt động".into(),
            ));
        }
        self.zone_repo.soft_delete(id).await?;
        Ok(())
    }
}
