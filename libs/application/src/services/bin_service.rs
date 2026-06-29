use std::str::FromStr;

use uuid::Uuid;
use validator::Validate;

use crate::dto::bin_dto::{BinResponse, CreateBinRequest, ListBinsQuery, UpdateBinRequest};
use crate::dto::pagination::Paginated;
use crate::errors::AppError;
use domain::entities::{BinFilter, BinSort, BinSortField, BinUpdate, NewBin, SortDir};
use crate::ports::{BinRepository, ZoneRepository};

const DEFAULT_PAGE_SIZE: u32 = 20;
const MAX_PAGE_SIZE: u32 = 100;

/// Chuẩn hoá chuỗi: trim, rỗng → None (không lọc).
fn norm(s: Option<String>) -> Option<String> {
    s.map(|v| v.trim().to_string()).filter(|v| !v.is_empty())
}

/// Parse chuỗi sort `field:dir,field:dir` → Vec<BinSort> (thiếu hướng → asc); lỗi → 400.
fn parse_sort(raw: &str) -> Result<Vec<BinSort>, AppError> {
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
        let field = BinSortField::from_str(&f.to_lowercase())
            .map_err(|_| AppError::Validation(format!("Trường sắp xếp không hợp lệ: {f}")))?;
        let dir = SortDir::from_str(&d.to_lowercase())
            .map_err(|_| AppError::Validation(format!("Hướng sắp xếp không hợp lệ: {d}")))?;
        out.push(BinSort { field, dir });
    }
    Ok(out)
}

pub struct BinService<BR, ZR>
where
    BR: BinRepository,
    ZR: ZoneRepository,
{
    bin_repo: BR,
    zone_repo: ZR,
}

impl<BR, ZR> BinService<BR, ZR>
where
    BR: BinRepository,
    ZR: ZoneRepository,
{
    pub fn new(bin_repo: BR, zone_repo: ZR) -> Self {
        Self { bin_repo, zone_repo }
    }

    /// Khu vực cha phải tồn tại & chưa xoá mềm (FK không bắt được cha đã xoá mềm).
    async fn ensure_zone_active(&self, zone_id: Uuid) -> Result<(), AppError> {
        self.zone_repo
            .find_by_id(zone_id)
            .await?
            .ok_or_else(|| AppError::Validation("Khu vực không tồn tại".into()))?;
        Ok(())
    }

    /// Tạo kệ mới (cần BIN_CREATE — kiểm ở middleware).
    pub async fn create_bin(&self, req: CreateBinRequest) -> Result<BinResponse, AppError> {
        req.validate()
            .map_err(|e| AppError::Validation(e.to_string()))?;
        self.ensure_zone_active(req.zone_id).await?;

        let bin = self
            .bin_repo
            .create(NewBin {
                zone_id: req.zone_id,
                code: req.code.trim().to_string(),
                name: req.name.trim().to_string(),
            })
            .await?;
        Ok(BinResponse::from(bin))
    }

    /// Danh sách kệ (lọc + phân trang + sắp xếp) (cần BIN_VIEW).
    pub async fn list_bins(&self, q: ListBinsQuery) -> Result<Paginated<BinResponse>, AppError> {
        let page = q.page.unwrap_or(1).max(1);
        let page_size = q.page_size.unwrap_or(DEFAULT_PAGE_SIZE).clamp(1, MAX_PAGE_SIZE);

        let sort = match q.sort.as_deref() {
            Some(s) => parse_sort(s)?,
            None => Vec::new(),
        };

        let filter = BinFilter {
            zone_id: q.zone_id,
            code: norm(q.code),
            name: norm(q.name),
            sort,
            limit: page_size as i64,
            offset: ((page - 1) * page_size) as i64,
        };

        let (bins, total) = self.bin_repo.list(filter).await?;

        let total_pages = if total == 0 {
            0
        } else {
            ((total as u64).div_ceil(page_size as u64)) as u32
        };

        Ok(Paginated {
            items: bins.into_iter().map(BinResponse::from).collect(),
            page,
            page_size,
            total_items: total,
            total_pages,
            has_next: page < total_pages,
            has_prev: page > 1,
        })
    }

    /// Chi tiết một kệ (cần BIN_VIEW).
    pub async fn get_bin(&self, id: Uuid) -> Result<BinResponse, AppError> {
        let bin = self
            .bin_repo
            .find_by_id(id)
            .await?
            .ok_or_else(|| AppError::NotFound("Kệ không tồn tại".into()))?;
        Ok(BinResponse::from(bin))
    }

    /// Cập nhật kệ (cần BIN_UPDATE). Cho đổi khu vực cha (kiểm cha mới còn hoạt động).
    pub async fn update_bin(
        &self,
        id: Uuid,
        req: UpdateBinRequest,
    ) -> Result<BinResponse, AppError> {
        req.validate()
            .map_err(|e| AppError::Validation(e.to_string()))?;

        if let Some(zone_id) = req.zone_id {
            self.ensure_zone_active(zone_id).await?;
        }

        let changes = BinUpdate {
            zone_id: req.zone_id,
            code: req.code.map(|c| c.trim().to_string()),
            name: req.name.map(|n| n.trim().to_string()),
        };

        let updated = self
            .bin_repo
            .update(id, changes)
            .await?
            .ok_or_else(|| AppError::NotFound("Kệ không tồn tại".into()))?;
        Ok(BinResponse::from(updated))
    }

    /// Xoá mềm kệ (cần BIN_DELETE). Kệ là lá — không có con để chặn.
    pub async fn delete_bin(&self, id: Uuid) -> Result<(), AppError> {
        self.bin_repo.soft_delete(id).await?;
        Ok(())
    }
}
