use domain::entities::{
    Location, LocationFilter, LocationSort, LocationSortField, LocationUpdate, NewLocation, SortDir,
};
use application::errors::AppError;
use application::ports::LocationRepository;
use sqlx::PgPool;
use sqlx::types::chrono::{DateTime, Utc};
use uuid::Uuid;

const LOCATION_COLS: &str = "id, code, name, address, created_at, updated_at";

/// Mệnh đề lọc dùng chung cho count + list (bỏ kho đã xoá mềm). Lọc optional bằng
/// `($n::text IS NULL OR col = $n)` — cast `::text` để Postgres biết kiểu khi NULL.
const LOCATION_LIST_WHERE: &str = r#"
    WHERE deleted_at IS NULL
      AND ($1::text IS NULL OR code = $1)
      AND ($2::text IS NULL OR name = $2)
"#;

fn map_sqlx_err(e: sqlx::Error) -> AppError {
    if let sqlx::Error::Database(db) = &e
        && db.is_unique_violation()
    {
        return AppError::Validation("Mã kho đã tồn tại".into());
    }
    AppError::Internal(e.to_string())
}

/// Dựng ORDER BY từ sort (cột + hướng đều literal từ match → an toàn injection).
/// Luôn nối tiebreaker `id DESC`; rỗng → mặc định created_at DESC.
fn order_by_clause(sort: &[LocationSort]) -> String {
    let mut parts: Vec<String> = sort
        .iter()
        .map(|s| {
            let col = match s.field {
                LocationSortField::Code => "code",
                LocationSortField::Name => "name",
                LocationSortField::CreatedAt => "created_at",
                LocationSortField::UpdatedAt => "updated_at",
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
struct LocationRow {
    id: Uuid,
    code: String,
    name: String,
    address: Option<String>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl From<LocationRow> for Location {
    fn from(r: LocationRow) -> Self {
        Self {
            id: r.id,
            code: r.code,
            name: r.name,
            address: r.address,
            created_at: r.created_at,
            updated_at: r.updated_at,
        }
    }
}

#[derive(Clone)]
pub struct PgLocationRepository {
    pool: PgPool,
}

impl PgLocationRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

impl LocationRepository for PgLocationRepository {
    async fn create(&self, new_location: NewLocation) -> Result<Location, AppError> {
        let sql = format!(
            "INSERT INTO locations (code, name, address) VALUES ($1, $2, $3) RETURNING {LOCATION_COLS}"
        );
        let row: LocationRow = sqlx::query_as(&sql)
            .bind(&new_location.code)
            .bind(&new_location.name)
            .bind(&new_location.address)
            .fetch_one(&self.pool)
            .await
            .map_err(map_sqlx_err)?; // unique_violation → "Mã kho đã tồn tại"
        Ok(Location::from(row))
    }

    async fn find_by_id(&self, id: Uuid) -> Result<Option<Location>, AppError> {
        let sql =
            format!("SELECT {LOCATION_COLS} FROM locations WHERE id = $1 AND deleted_at IS NULL");
        let row: Option<LocationRow> = sqlx::query_as(&sql)
            .bind(id)
            .fetch_optional(&self.pool)
            .await
            .map_err(map_sqlx_err)?;
        Ok(row.map(Location::from))
    }

    async fn list(&self, filter: LocationFilter) -> Result<(Vec<Location>, i64), AppError> {
        let count_sql = format!("SELECT COUNT(*) FROM locations {LOCATION_LIST_WHERE}");
        let total: i64 = sqlx::query_scalar(&count_sql)
            .bind(&filter.code)
            .bind(&filter.name)
            .fetch_one(&self.pool)
            .await
            .map_err(map_sqlx_err)?;

        let order = order_by_clause(&filter.sort);
        let list_sql = format!(
            "SELECT {LOCATION_COLS} FROM locations {LOCATION_LIST_WHERE} {order} LIMIT $3 OFFSET $4"
        );
        let rows: Vec<LocationRow> = sqlx::query_as(&list_sql)
            .bind(&filter.code)
            .bind(&filter.name)
            .bind(filter.limit)
            .bind(filter.offset)
            .fetch_all(&self.pool)
            .await
            .map_err(map_sqlx_err)?;

        Ok((rows.into_iter().map(Location::from).collect(), total))
    }

    async fn update(
        &self,
        id: Uuid,
        changes: LocationUpdate,
    ) -> Result<Option<Location>, AppError> {
        let sql = format!(
            r#"
            UPDATE locations SET
                code = COALESCE($2, code),
                name = COALESCE($3, name),
                address = COALESCE($4, address)
            WHERE id = $1 AND deleted_at IS NULL
            RETURNING {LOCATION_COLS}
            "#
        );
        let row: Option<LocationRow> = sqlx::query_as(&sql)
            .bind(id)
            .bind(&changes.code)
            .bind(&changes.name)
            .bind(&changes.address)
            .fetch_optional(&self.pool)
            .await
            .map_err(map_sqlx_err)?; // unique_violation → "Mã kho đã tồn tại"
        Ok(row.map(Location::from))
    }

    async fn soft_delete(&self, id: Uuid) -> Result<(), AppError> {
        let res =
            sqlx::query("UPDATE locations SET deleted_at = NOW() WHERE id = $1 AND deleted_at IS NULL")
                .bind(id)
                .execute(&self.pool)
                .await
                .map_err(map_sqlx_err)?;
        if res.rows_affected() == 0 {
            return Err(AppError::NotFound("Kho không tồn tại".into()));
        }
        Ok(())
    }

    async fn has_active_zones(&self, location_id: Uuid) -> Result<bool, AppError> {
        let exists: bool = sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM warehouse_zones WHERE location_id = $1 AND deleted_at IS NULL)",
        )
        .bind(location_id)
        .fetch_one(&self.pool)
        .await
        .map_err(map_sqlx_err)?;
        Ok(exists)
    }
}
