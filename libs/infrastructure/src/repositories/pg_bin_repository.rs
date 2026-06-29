use domain::entities::{Bin, BinFilter, BinSort, BinSortField, BinUpdate, NewBin, SortDir};
use application::errors::AppError;
use application::ports::BinRepository;
use sqlx::PgPool;
use sqlx::types::chrono::{DateTime, Utc};
use uuid::Uuid;

const BIN_COLS: &str = "id, zone_id, code, name, created_at, updated_at";

const BIN_LIST_WHERE: &str = r#"
    WHERE deleted_at IS NULL
      AND ($1::uuid IS NULL OR zone_id = $1)
      AND ($2::text IS NULL OR code = $2)
      AND ($3::text IS NULL OR name = $3)
"#;

/// Map lỗi DB → AppError theo TÊN constraint/index (PG trả tên cả với unique index).
fn map_sqlx_err(e: sqlx::Error) -> AppError {
    if let sqlx::Error::Database(db) = &e {
        match db.constraint() {
            Some("uq_storage_bins_code") => {
                return AppError::Validation("Mã kệ đã tồn tại".into());
            }
            Some("uq_storage_bins_zone_name") => {
                return AppError::Validation("Tên kệ đã tồn tại trong khu vực".into());
            }
            _ => {}
        }
        if db.is_foreign_key_violation() {
            return AppError::Validation("Khu vực không tồn tại".into());
        }
    }
    AppError::Internal(e.to_string())
}

fn order_by_clause(sort: &[BinSort]) -> String {
    let mut parts: Vec<String> = sort
        .iter()
        .map(|s| {
            let col = match s.field {
                BinSortField::Code => "code",
                BinSortField::Name => "name",
                BinSortField::CreatedAt => "created_at",
                BinSortField::UpdatedAt => "updated_at",
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
struct BinRow {
    id: Uuid,
    zone_id: Uuid,
    code: String,
    name: String,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl From<BinRow> for Bin {
    fn from(r: BinRow) -> Self {
        Self {
            id: r.id,
            zone_id: r.zone_id,
            code: r.code,
            name: r.name,
            created_at: r.created_at,
            updated_at: r.updated_at,
        }
    }
}

#[derive(Clone)]
pub struct PgBinRepository {
    pool: PgPool,
}

impl PgBinRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

impl BinRepository for PgBinRepository {
    async fn create(&self, new_bin: NewBin) -> Result<Bin, AppError> {
        let sql = format!(
            "INSERT INTO storage_bins (zone_id, code, name) VALUES ($1, $2, $3) RETURNING {BIN_COLS}"
        );
        let row: BinRow = sqlx::query_as(&sql)
            .bind(new_bin.zone_id)
            .bind(&new_bin.code)
            .bind(&new_bin.name)
            .fetch_one(&self.pool)
            .await
            .map_err(map_sqlx_err)?;
        Ok(Bin::from(row))
    }

    async fn find_by_id(&self, id: Uuid) -> Result<Option<Bin>, AppError> {
        let sql =
            format!("SELECT {BIN_COLS} FROM storage_bins WHERE id = $1 AND deleted_at IS NULL");
        let row: Option<BinRow> = sqlx::query_as(&sql)
            .bind(id)
            .fetch_optional(&self.pool)
            .await
            .map_err(map_sqlx_err)?;
        Ok(row.map(Bin::from))
    }

    async fn list(&self, filter: BinFilter) -> Result<(Vec<Bin>, i64), AppError> {
        let count_sql = format!("SELECT COUNT(*) FROM storage_bins {BIN_LIST_WHERE}");
        let total: i64 = sqlx::query_scalar(&count_sql)
            .bind(filter.zone_id)
            .bind(&filter.code)
            .bind(&filter.name)
            .fetch_one(&self.pool)
            .await
            .map_err(map_sqlx_err)?;

        let order = order_by_clause(&filter.sort);
        let list_sql = format!(
            "SELECT {BIN_COLS} FROM storage_bins {BIN_LIST_WHERE} {order} LIMIT $4 OFFSET $5"
        );
        let rows: Vec<BinRow> = sqlx::query_as(&list_sql)
            .bind(filter.zone_id)
            .bind(&filter.code)
            .bind(&filter.name)
            .bind(filter.limit)
            .bind(filter.offset)
            .fetch_all(&self.pool)
            .await
            .map_err(map_sqlx_err)?;

        Ok((rows.into_iter().map(Bin::from).collect(), total))
    }

    async fn update(&self, id: Uuid, changes: BinUpdate) -> Result<Option<Bin>, AppError> {
        let sql = format!(
            r#"
            UPDATE storage_bins SET
                zone_id = COALESCE($2, zone_id),
                code = COALESCE($3, code),
                name = COALESCE($4, name)
            WHERE id = $1 AND deleted_at IS NULL
            RETURNING {BIN_COLS}
            "#
        );
        let row: Option<BinRow> = sqlx::query_as(&sql)
            .bind(id)
            .bind(changes.zone_id)
            .bind(&changes.code)
            .bind(&changes.name)
            .fetch_optional(&self.pool)
            .await
            .map_err(map_sqlx_err)?;
        Ok(row.map(Bin::from))
    }

    async fn soft_delete(&self, id: Uuid) -> Result<(), AppError> {
        let res = sqlx::query(
            "UPDATE storage_bins SET deleted_at = NOW() WHERE id = $1 AND deleted_at IS NULL",
        )
        .bind(id)
        .execute(&self.pool)
        .await
        .map_err(map_sqlx_err)?;
        if res.rows_affected() == 0 {
            return Err(AppError::NotFound("Kệ không tồn tại".into()));
        }
        Ok(())
    }
}
