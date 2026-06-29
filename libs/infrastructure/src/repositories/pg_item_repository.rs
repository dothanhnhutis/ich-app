use std::str::FromStr;

use domain::entities::{
    Item, ItemFilter, ItemSort, ItemSortField, ItemType, ItemUpdate, NewItem, PackagingLevel,
    SortDir,
};
use application::errors::AppError;
use application::ports::ItemRepository;
use sqlx::PgPool;
use sqlx::types::chrono::{DateTime, Utc};
use uuid::Uuid;

/// `type` là từ khoá Rust → field `item_type` (sqlx rename). Cột số = DOUBLE PRECISION (f64 native).
const ITEM_COLS: &str = "id, sku, name, type, base_uom, packaging_level, is_purchasable, \
    is_sellable, has_bom, is_lot_controlled, is_phantom, density_g_ml, \
    shelf_life_days, pao_months, inci_name, cas_number, description, created_at, updated_at";

/// Lọc optional bằng `($n::type IS NULL OR col = $n)`. `$4` giới hạn theo quyền per-type.
const ITEM_LIST_WHERE: &str = r#"
    WHERE deleted_at IS NULL
      AND ($1::text IS NULL OR sku = $1)
      AND ($2::text IS NULL OR name = $2)
      AND ($3::text IS NULL OR type = $3)
      AND ($4::text[] IS NULL OR type = ANY($4))
"#;

fn map_sqlx_err(e: sqlx::Error) -> AppError {
    if let sqlx::Error::Database(db) = &e {
        match db.constraint() {
            Some("uq_items_sku") => {
                return AppError::Validation("SKU vật tư đã tồn tại".into());
            }
            Some("chk_items_type") => {
                return AppError::Validation("Loại vật tư không hợp lệ".into());
            }
            Some("chk_items_pkg_level") => {
                return AppError::Validation(
                    "Cấp bao bì chỉ dành cho vật tư loại PACKAGING (và bắt buộc khi PACKAGING)".into(),
                );
            }
            Some("chk_items_phantom") => {
                return AppError::Validation("Item phantom bắt buộc phải có BOM".into());
            }
            Some("chk_items_density") => {
                return AppError::Validation("Khối lượng riêng phải lớn hơn 0".into());
            }
            Some("chk_items_shelf_life") => {
                return AppError::Validation("Hạn sử dụng (ngày) phải lớn hơn 0".into());
            }
            Some("chk_items_pao") => {
                return AppError::Validation("PAO (tháng) phải lớn hơn 0".into());
            }
            _ => {}
        }
    }
    AppError::Internal(e.to_string())
}

fn order_by_clause(sort: &[ItemSort]) -> String {
    let mut parts: Vec<String> = sort
        .iter()
        .map(|s| {
            let col = match s.field {
                ItemSortField::Sku => "sku",
                ItemSortField::Name => "name",
                ItemSortField::ItemType => "type",
                ItemSortField::CreatedAt => "created_at",
                ItemSortField::UpdatedAt => "updated_at",
            };
            let dir = match s.dir {
                SortDir::Asc => "ASC",
                SortDir::Desc => "DESC",
            };
            format!("{col} {dir}")
        })
        .collect();
    if parts.is_empty() {
        parts.push("created_at DESC".into());
    }
    parts.push("id DESC".into());
    format!("ORDER BY {}", parts.join(", "))
}

#[derive(sqlx::FromRow)]
struct ItemRow {
    id: Uuid,
    sku: String,
    name: String,
    #[sqlx(rename = "type")]
    item_type: String,
    base_uom: String,
    packaging_level: Option<String>,
    is_purchasable: bool,
    is_sellable: bool,
    has_bom: bool,
    is_lot_controlled: bool,
    is_phantom: bool,
    density_g_ml: Option<f64>,
    shelf_life_days: Option<i32>,
    pao_months: Option<i16>,
    inci_name: Option<String>,
    cas_number: Option<String>,
    description: Option<String>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl TryFrom<ItemRow> for Item {
    type Error = AppError;
    fn try_from(r: ItemRow) -> Result<Self, Self::Error> {
        let packaging_level = match r.packaging_level {
            Some(s) => Some(PackagingLevel::from_str(&s).map_err(AppError::Internal)?),
            None => None,
        };
        Ok(Item {
            id: r.id,
            sku: r.sku,
            name: r.name,
            item_type: ItemType::from_str(&r.item_type).map_err(AppError::Internal)?,
            base_uom: r.base_uom,
            packaging_level,
            is_purchasable: r.is_purchasable,
            is_sellable: r.is_sellable,
            has_bom: r.has_bom,
            is_lot_controlled: r.is_lot_controlled,
            is_phantom: r.is_phantom,
            density_g_ml: r.density_g_ml,
            shelf_life_days: r.shelf_life_days,
            pao_months: r.pao_months,
            inci_name: r.inci_name,
            cas_number: r.cas_number,
            description: r.description,
            created_at: r.created_at,
            updated_at: r.updated_at,
        })
    }
}

#[derive(Clone)]
pub struct PgItemRepository {
    pool: PgPool,
}

impl PgItemRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

impl ItemRepository for PgItemRepository {
    async fn create(&self, new_item: NewItem) -> Result<Item, AppError> {
        let sql = format!(
            r#"
            INSERT INTO items
                (sku, name, type, base_uom, packaging_level, is_purchasable, is_sellable,
                 has_bom, is_lot_controlled, is_phantom, density_g_ml, shelf_life_days,
                 pao_months, inci_name, cas_number, description)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16)
            RETURNING {ITEM_COLS}
            "#
        );
        let row: ItemRow = sqlx::query_as(&sql)
            .bind(&new_item.sku)
            .bind(&new_item.name)
            .bind(new_item.item_type.as_str())
            .bind(&new_item.base_uom)
            .bind(new_item.packaging_level.map(|p| p.as_str()))
            .bind(new_item.is_purchasable)
            .bind(new_item.is_sellable)
            .bind(new_item.has_bom)
            .bind(new_item.is_lot_controlled)
            .bind(new_item.is_phantom)
            .bind(new_item.density_g_ml)
            .bind(new_item.shelf_life_days)
            .bind(new_item.pao_months)
            .bind(&new_item.inci_name)
            .bind(&new_item.cas_number)
            .bind(&new_item.description)
            .fetch_one(&self.pool)
            .await
            .map_err(map_sqlx_err)?;
        Item::try_from(row)
    }

    async fn find_by_id(&self, id: Uuid) -> Result<Option<Item>, AppError> {
        let sql = format!("SELECT {ITEM_COLS} FROM items WHERE id = $1 AND deleted_at IS NULL");
        let row: Option<ItemRow> = sqlx::query_as(&sql)
            .bind(id)
            .fetch_optional(&self.pool)
            .await
            .map_err(map_sqlx_err)?;
        row.map(Item::try_from).transpose()
    }

    async fn list(&self, filter: ItemFilter) -> Result<(Vec<Item>, i64), AppError> {
        let count_sql = format!("SELECT COUNT(*) FROM items {ITEM_LIST_WHERE}");
        let total: i64 = sqlx::query_scalar(&count_sql)
            .bind(&filter.sku)
            .bind(&filter.name)
            .bind(&filter.item_type)
            .bind(&filter.allowed_types)
            .fetch_one(&self.pool)
            .await
            .map_err(map_sqlx_err)?;

        let order = order_by_clause(&filter.sort);
        let list_sql =
            format!("SELECT {ITEM_COLS} FROM items {ITEM_LIST_WHERE} {order} LIMIT $5 OFFSET $6");
        let rows: Vec<ItemRow> = sqlx::query_as(&list_sql)
            .bind(&filter.sku)
            .bind(&filter.name)
            .bind(&filter.item_type)
            .bind(&filter.allowed_types)
            .bind(filter.limit)
            .bind(filter.offset)
            .fetch_all(&self.pool)
            .await
            .map_err(map_sqlx_err)?;

        let items = rows
            .into_iter()
            .map(Item::try_from)
            .collect::<Result<Vec<_>, _>>()?;
        Ok((items, total))
    }

    async fn update(&self, id: Uuid, changes: ItemUpdate) -> Result<Option<Item>, AppError> {
        let packaging_level = changes.packaging_level.map(|p| p.as_str());
        // `type` & `base_uom` bị khoá → không nằm trong SET.
        let sql = format!(
            r#"
            UPDATE items SET
                sku = COALESCE($2, sku),
                name = COALESCE($3, name),
                packaging_level = COALESCE($4, packaging_level),
                is_purchasable = COALESCE($5, is_purchasable),
                is_sellable = COALESCE($6, is_sellable),
                has_bom = COALESCE($7, has_bom),
                is_lot_controlled = COALESCE($8, is_lot_controlled),
                is_phantom = COALESCE($9, is_phantom),
                density_g_ml = COALESCE($10, density_g_ml),
                shelf_life_days = COALESCE($11, shelf_life_days),
                pao_months = COALESCE($12, pao_months),
                inci_name = COALESCE($13, inci_name),
                cas_number = COALESCE($14, cas_number),
                description = COALESCE($15, description)
            WHERE id = $1 AND deleted_at IS NULL
            RETURNING {ITEM_COLS}
            "#
        );
        let row: Option<ItemRow> = sqlx::query_as(&sql)
            .bind(id)
            .bind(&changes.sku)
            .bind(&changes.name)
            .bind(packaging_level)
            .bind(changes.is_purchasable)
            .bind(changes.is_sellable)
            .bind(changes.has_bom)
            .bind(changes.is_lot_controlled)
            .bind(changes.is_phantom)
            .bind(changes.density_g_ml)
            .bind(changes.shelf_life_days)
            .bind(changes.pao_months)
            .bind(&changes.inci_name)
            .bind(&changes.cas_number)
            .bind(&changes.description)
            .fetch_optional(&self.pool)
            .await
            .map_err(map_sqlx_err)?;
        row.map(Item::try_from).transpose()
    }

    async fn soft_delete(&self, id: Uuid) -> Result<(), AppError> {
        let res =
            sqlx::query("UPDATE items SET deleted_at = NOW() WHERE id = $1 AND deleted_at IS NULL")
                .bind(id)
                .execute(&self.pool)
                .await
                .map_err(map_sqlx_err)?;
        if res.rows_affected() == 0 {
            return Err(AppError::NotFound("Vật tư không tồn tại".into()));
        }
        Ok(())
    }

    async fn is_referenced(&self, item_id: Uuid) -> Result<bool, AppError> {
        let referenced: bool = sqlx::query_scalar(
            r#"
            SELECT
                EXISTS(SELECT 1 FROM boms WHERE output_item_id = $1 AND deleted_at IS NULL)
             OR EXISTS(SELECT 1 FROM bom_lines WHERE component_item_id = $1 AND deleted_at IS NULL)
             OR EXISTS(SELECT 1 FROM item_uom_conversions WHERE item_id = $1 AND deleted_at IS NULL)
             OR EXISTS(SELECT 1 FROM vendor_items WHERE item_id = $1)
            "#,
        )
        .bind(item_id)
        .fetch_one(&self.pool)
        .await
        .map_err(map_sqlx_err)?;
        Ok(referenced)
    }
}
