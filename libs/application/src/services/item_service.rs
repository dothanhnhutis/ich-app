use std::str::FromStr;

use uuid::Uuid;
use validator::Validate;

use crate::dto::item_dto::{CreateItemRequest, ItemResponse, ListItemsQuery, UpdateItemRequest};
use crate::dto::pagination::Paginated;
use crate::errors::AppError;
use domain::entities::{
    ItemFilter, ItemSort, ItemSortField, ItemType, ItemUpdate, NewItem, PackagingLevel, SortDir,
};
use domain::repositories::ItemRepository;

const DEFAULT_PAGE_SIZE: u32 = 20;
const MAX_PAGE_SIZE: u32 = 100;

/// 5 loại item (để suy ra tập quyền VIEW khi liệt kê).
const ALL_ITEM_TYPES: [&str; 5] = [
    "RAW_MATERIAL",
    "PACKAGING",
    "UTILITY",
    "SEMI_FINISHED",
    "FINISHED_GOODS",
];

/// Chuẩn hoá chuỗi: trim, rỗng → None.
fn norm(s: Option<String>) -> Option<String> {
    s.map(|v| v.trim().to_string()).filter(|v| !v.is_empty())
}

fn parse_item_type(s: &str) -> Result<ItemType, AppError> {
    ItemType::from_str(s.trim()).map_err(|_| AppError::Validation("Loại vật tư không hợp lệ".into()))
}

fn parse_pkg_level(s: &str) -> Result<PackagingLevel, AppError> {
    PackagingLevel::from_str(s.trim())
        .map_err(|_| AppError::Validation("Cấp bao bì không hợp lệ".into()))
}

/// Per-type authz: yêu cầu user có đúng quyền `{TYPE}_{ACTION}`.
fn require(codes: &[String], code: &str) -> Result<(), AppError> {
    if codes.iter().any(|c| c == code) {
        Ok(())
    } else {
        Err(AppError::Forbidden(
            "Bạn không có quyền thực hiện thao tác này".into(),
        ))
    }
}

/// Parse chuỗi sort `field:dir,field:dir` → Vec<ItemSort>; lỗi → 400.
fn parse_sort(raw: &str) -> Result<Vec<ItemSort>, AppError> {
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
        let field = ItemSortField::from_str(&f.to_lowercase())
            .map_err(|_| AppError::Validation(format!("Trường sắp xếp không hợp lệ: {f}")))?;
        let dir = SortDir::from_str(&d.to_lowercase())
            .map_err(|_| AppError::Validation(format!("Hướng sắp xếp không hợp lệ: {d}")))?;
        out.push(ItemSort { field, dir });
    }
    Ok(out)
}

/// Kiểm tính nhất quán packaging_level ↔ type (thông điệp rõ hơn CHECK của DB).
fn check_pkg_consistency(
    item_type: ItemType,
    pkg: Option<PackagingLevel>,
) -> Result<(), AppError> {
    match item_type {
        ItemType::Packaging if pkg.is_none() => Err(AppError::Validation(
            "Vật tư loại PACKAGING bắt buộc có packaging_level".into(),
        )),
        ItemType::Packaging => Ok(()),
        _ if pkg.is_some() => Err(AppError::Validation(
            "Chỉ vật tư loại PACKAGING mới có packaging_level".into(),
        )),
        _ => Ok(()),
    }
}

pub struct ItemService<IR>
where
    IR: ItemRepository,
{
    item_repo: IR,
}

impl<IR> ItemService<IR>
where
    IR: ItemRepository,
{
    pub fn new(item_repo: IR) -> Self {
        Self { item_repo }
    }

    /// Tạo vật tư (cần `{TYPE}_CREATE` theo loại trong body).
    pub async fn create_item(
        &self,
        codes: &[String],
        req: CreateItemRequest,
    ) -> Result<ItemResponse, AppError> {
        req.validate()
            .map_err(|e| AppError::Validation(e.to_string()))?;
        let item_type = parse_item_type(&req.item_type)?;
        require(codes, &format!("{}_CREATE", item_type.as_str()))?;

        let packaging_level = match req.packaging_level.as_deref() {
            Some(s) => Some(parse_pkg_level(s)?),
            None => None,
        };
        check_pkg_consistency(item_type, packaging_level)?;

        let item = self
            .item_repo
            .create(NewItem {
                sku: req.sku.trim().to_string(),
                name: req.name.trim().to_string(),
                item_type,
                base_uom: req.base_uom.trim().to_string(),
                packaging_level,
                is_purchasable: req.is_purchasable,
                is_sellable: req.is_sellable,
                has_bom: req.has_bom,
                is_lot_controlled: req.is_lot_controlled,
                is_phantom: req.is_phantom,
                density_g_ml: req.density_g_ml,
                shelf_life_days: req.shelf_life_days,
                pao_months: req.pao_months,
                inci_name: norm(req.inci_name),
                cas_number: norm(req.cas_number),
                description: norm(req.description),
            })
            .await?;
        Ok(ItemResponse::from(item))
    }

    /// Danh sách vật tư — chỉ trả về các loại user có `{TYPE}_VIEW`.
    pub async fn list_items(
        &self,
        codes: &[String],
        q: ListItemsQuery,
    ) -> Result<Paginated<ItemResponse>, AppError> {
        let allowed: Vec<String> = ALL_ITEM_TYPES
            .iter()
            .filter(|t| codes.iter().any(|c| c == &format!("{t}_VIEW")))
            .map(|t| t.to_string())
            .collect();
        if allowed.is_empty() {
            return Err(AppError::Forbidden(
                "Bạn không có quyền xem vật tư".into(),
            ));
        }

        let requested = norm(q.item_type);
        if let Some(t) = &requested {
            parse_item_type(t)?; // 400 nếu loại không hợp lệ
            if !allowed.contains(t) {
                return Err(AppError::Forbidden(
                    "Bạn không có quyền xem loại vật tư này".into(),
                ));
            }
        }

        let page = q.page.unwrap_or(1).max(1);
        let page_size = q.page_size.unwrap_or(DEFAULT_PAGE_SIZE).clamp(1, MAX_PAGE_SIZE);
        let sort = match q.sort.as_deref() {
            Some(s) => parse_sort(s)?,
            None => Vec::new(),
        };

        let filter = ItemFilter {
            sku: norm(q.sku),
            name: norm(q.name),
            item_type: requested,
            allowed_types: Some(allowed),
            sort,
            limit: page_size as i64,
            offset: ((page - 1) * page_size) as i64,
        };

        let (items, total) = self.item_repo.list(filter).await?;

        let total_pages = if total == 0 {
            0
        } else {
            ((total as u64).div_ceil(page_size as u64)) as u32
        };

        Ok(Paginated {
            items: items.into_iter().map(ItemResponse::from).collect(),
            page,
            page_size,
            total_items: total,
            total_pages,
            has_next: page < total_pages,
            has_prev: page > 1,
        })
    }

    /// Chi tiết vật tư (cần `{TYPE}_VIEW` theo loại của item).
    pub async fn get_item(&self, codes: &[String], id: Uuid) -> Result<ItemResponse, AppError> {
        let item = self
            .item_repo
            .find_by_id(id)
            .await?
            .ok_or_else(|| AppError::NotFound("Vật tư không tồn tại".into()))?;
        require(codes, &format!("{}_VIEW", item.item_type.as_str()))?;
        Ok(ItemResponse::from(item))
    }

    /// Cập nhật vật tư (cần `{TYPE}_UPDATE`). `type`/`base_uom` bị khoá.
    pub async fn update_item(
        &self,
        codes: &[String],
        id: Uuid,
        req: UpdateItemRequest,
    ) -> Result<ItemResponse, AppError> {
        req.validate()
            .map_err(|e| AppError::Validation(e.to_string()))?;

        let item = self
            .item_repo
            .find_by_id(id)
            .await?
            .ok_or_else(|| AppError::NotFound("Vật tư không tồn tại".into()))?;
        require(codes, &format!("{}_UPDATE", item.item_type.as_str()))?;

        let packaging_level = match req.packaging_level.as_deref() {
            Some(s) => Some(parse_pkg_level(s)?),
            None => None,
        };
        if packaging_level.is_some() && item.item_type != ItemType::Packaging {
            return Err(AppError::Validation(
                "Chỉ vật tư loại PACKAGING mới có packaging_level".into(),
            ));
        }

        let changes = ItemUpdate {
            sku: req.sku.map(|s| s.trim().to_string()),
            name: req.name.map(|n| n.trim().to_string()),
            packaging_level,
            is_purchasable: req.is_purchasable,
            is_sellable: req.is_sellable,
            has_bom: req.has_bom,
            is_lot_controlled: req.is_lot_controlled,
            is_phantom: req.is_phantom,
            density_g_ml: req.density_g_ml,
            shelf_life_days: req.shelf_life_days,
            pao_months: req.pao_months,
            inci_name: norm(req.inci_name),
            cas_number: norm(req.cas_number),
            description: norm(req.description),
        };

        let updated = self
            .item_repo
            .update(id, changes)
            .await?
            .ok_or_else(|| AppError::NotFound("Vật tư không tồn tại".into()))?;
        Ok(ItemResponse::from(updated))
    }

    /// Xoá mềm vật tư (cần `{TYPE}_DELETE`); chặn nếu còn tham chiếu active.
    pub async fn delete_item(&self, codes: &[String], id: Uuid) -> Result<(), AppError> {
        let item = self
            .item_repo
            .find_by_id(id)
            .await?
            .ok_or_else(|| AppError::NotFound("Vật tư không tồn tại".into()))?;
        require(codes, &format!("{}_DELETE", item.item_type.as_str()))?;

        if self.item_repo.is_referenced(id).await? {
            return Err(AppError::Validation(
                "Vật tư đang được tham chiếu (BOM / đơn vị quy đổi / nhà cung cấp), không thể xoá"
                    .into(),
            ));
        }
        self.item_repo.soft_delete(id).await?;
        Ok(())
    }
}
