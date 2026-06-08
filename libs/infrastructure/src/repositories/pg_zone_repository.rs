use std::str::FromStr;

use domain::entities::{
    NewZone, SortDir, Zone, ZoneFilter, ZoneSort, ZoneSortField, ZoneType, ZoneUpdate,
};
use domain::errors::DomainError;
use domain::repositories::ZoneRepository;
use sqlx::PgPool;
use sqlx::types::chrono::{DateTime, Utc};
use uuid::Uuid;

const ZONE_COLS: &str = "id, location_id, name, zone_type, temp_min_c, temp_max_c, \
    humidity_max_pct, is_light_protected, is_ventilated, is_explosion_proof, created_at, updated_at";

/// Lọc optional bằng `($n::type IS NULL OR col = $n)` — cast để Postgres biết kiểu khi NULL.
const ZONE_LIST_WHERE: &str = r#"
    WHERE deleted_at IS NULL
      AND ($1::uuid IS NULL OR location_id = $1)
      AND ($2::text IS NULL OR name = $2)
      AND ($3::text IS NULL OR zone_type = $3)
"#;

/// Map lỗi DB → DomainError theo TÊN constraint/index (PG trả tên cả với unique index).
fn map_sqlx_err(e: sqlx::Error) -> DomainError {
    if let sqlx::Error::Database(db) = &e {
        match db.constraint() {
            Some("uq_warehouse_zones_loc_name") => {
                return DomainError::AlreadyExists("Tên khu vực đã tồn tại trong kho".into());
            }
            Some("chk_warehouse_zones_temp_range") => {
                return DomainError::Validation(
                    "Nhiệt độ tối thiểu phải nhỏ hơn hoặc bằng tối đa".into(),
                );
            }
            Some("chk_warehouse_zones_humidity") => {
                return DomainError::Validation("Độ ẩm phải trong khoảng 0–100".into());
            }
            Some("chk_warehouse_zones_type") => {
                return DomainError::Validation("Loại khu vực không hợp lệ".into());
            }
            _ => {}
        }
        if db.is_foreign_key_violation() {
            return DomainError::Validation("Kho không tồn tại".into());
        }
    }
    DomainError::Internal(e.to_string())
}

fn order_by_clause(sort: &[ZoneSort]) -> String {
    let mut parts: Vec<String> = sort
        .iter()
        .map(|s| {
            let col = match s.field {
                ZoneSortField::Name => "name",
                ZoneSortField::ZoneType => "zone_type",
                ZoneSortField::CreatedAt => "created_at",
                ZoneSortField::UpdatedAt => "updated_at",
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
struct ZoneRow {
    id: Uuid,
    location_id: Uuid,
    name: String,
    zone_type: String,
    temp_min_c: Option<f64>,
    temp_max_c: Option<f64>,
    humidity_max_pct: Option<f64>,
    is_light_protected: bool,
    is_ventilated: bool,
    is_explosion_proof: bool,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl TryFrom<ZoneRow> for Zone {
    type Error = DomainError;
    fn try_from(r: ZoneRow) -> Result<Self, Self::Error> {
        Ok(Zone {
            id: r.id,
            location_id: r.location_id,
            name: r.name,
            zone_type: ZoneType::from_str(&r.zone_type).map_err(DomainError::Internal)?,
            temp_min_c: r.temp_min_c,
            temp_max_c: r.temp_max_c,
            humidity_max_pct: r.humidity_max_pct,
            is_light_protected: r.is_light_protected,
            is_ventilated: r.is_ventilated,
            is_explosion_proof: r.is_explosion_proof,
            created_at: r.created_at,
            updated_at: r.updated_at,
        })
    }
}

#[derive(Clone)]
pub struct PgZoneRepository {
    pool: PgPool,
}

impl PgZoneRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

impl ZoneRepository for PgZoneRepository {
    async fn create(&self, new_zone: NewZone) -> Result<Zone, DomainError> {
        let sql = format!(
            r#"
            INSERT INTO warehouse_zones
                (location_id, name, zone_type, temp_min_c, temp_max_c, humidity_max_pct,
                 is_light_protected, is_ventilated, is_explosion_proof)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            RETURNING {ZONE_COLS}
            "#
        );
        let row: ZoneRow = sqlx::query_as(&sql)
            .bind(new_zone.location_id)
            .bind(&new_zone.name)
            .bind(new_zone.zone_type.as_str())
            .bind(new_zone.temp_min_c)
            .bind(new_zone.temp_max_c)
            .bind(new_zone.humidity_max_pct)
            .bind(new_zone.is_light_protected)
            .bind(new_zone.is_ventilated)
            .bind(new_zone.is_explosion_proof)
            .fetch_one(&self.pool)
            .await
            .map_err(map_sqlx_err)?;
        Zone::try_from(row)
    }

    async fn find_by_id(&self, id: Uuid) -> Result<Option<Zone>, DomainError> {
        let sql =
            format!("SELECT {ZONE_COLS} FROM warehouse_zones WHERE id = $1 AND deleted_at IS NULL");
        let row: Option<ZoneRow> = sqlx::query_as(&sql)
            .bind(id)
            .fetch_optional(&self.pool)
            .await
            .map_err(map_sqlx_err)?;
        row.map(Zone::try_from).transpose()
    }

    async fn list(&self, filter: ZoneFilter) -> Result<(Vec<Zone>, i64), DomainError> {
        let count_sql = format!("SELECT COUNT(*) FROM warehouse_zones {ZONE_LIST_WHERE}");
        let total: i64 = sqlx::query_scalar(&count_sql)
            .bind(filter.location_id)
            .bind(&filter.name)
            .bind(&filter.zone_type)
            .fetch_one(&self.pool)
            .await
            .map_err(map_sqlx_err)?;

        let order = order_by_clause(&filter.sort);
        let list_sql = format!(
            "SELECT {ZONE_COLS} FROM warehouse_zones {ZONE_LIST_WHERE} {order} LIMIT $4 OFFSET $5"
        );
        let rows: Vec<ZoneRow> = sqlx::query_as(&list_sql)
            .bind(filter.location_id)
            .bind(&filter.name)
            .bind(&filter.zone_type)
            .bind(filter.limit)
            .bind(filter.offset)
            .fetch_all(&self.pool)
            .await
            .map_err(map_sqlx_err)?;

        let zones = rows
            .into_iter()
            .map(Zone::try_from)
            .collect::<Result<Vec<_>, _>>()?;
        Ok((zones, total))
    }

    async fn update(&self, id: Uuid, changes: ZoneUpdate) -> Result<Option<Zone>, DomainError> {
        let zone_type = changes.zone_type.map(|z| z.as_str());
        let sql = format!(
            r#"
            UPDATE warehouse_zones SET
                location_id = COALESCE($2, location_id),
                name = COALESCE($3, name),
                zone_type = COALESCE($4, zone_type),
                temp_min_c = COALESCE($5, temp_min_c),
                temp_max_c = COALESCE($6, temp_max_c),
                humidity_max_pct = COALESCE($7, humidity_max_pct),
                is_light_protected = COALESCE($8, is_light_protected),
                is_ventilated = COALESCE($9, is_ventilated),
                is_explosion_proof = COALESCE($10, is_explosion_proof)
            WHERE id = $1 AND deleted_at IS NULL
            RETURNING {ZONE_COLS}
            "#
        );
        let row: Option<ZoneRow> = sqlx::query_as(&sql)
            .bind(id)
            .bind(changes.location_id)
            .bind(&changes.name)
            .bind(zone_type)
            .bind(changes.temp_min_c)
            .bind(changes.temp_max_c)
            .bind(changes.humidity_max_pct)
            .bind(changes.is_light_protected)
            .bind(changes.is_ventilated)
            .bind(changes.is_explosion_proof)
            .fetch_optional(&self.pool)
            .await
            .map_err(map_sqlx_err)?;
        row.map(Zone::try_from).transpose()
    }

    async fn soft_delete(&self, id: Uuid) -> Result<(), DomainError> {
        let res = sqlx::query(
            "UPDATE warehouse_zones SET deleted_at = NOW() WHERE id = $1 AND deleted_at IS NULL",
        )
        .bind(id)
        .execute(&self.pool)
        .await
        .map_err(map_sqlx_err)?;
        if res.rows_affected() == 0 {
            return Err(DomainError::NotFound("Khu vực không tồn tại".into()));
        }
        Ok(())
    }

    async fn has_active_bins(&self, zone_id: Uuid) -> Result<bool, DomainError> {
        let exists: bool = sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM storage_bins WHERE zone_id = $1 AND deleted_at IS NULL)",
        )
        .bind(zone_id)
        .fetch_one(&self.pool)
        .await
        .map_err(map_sqlx_err)?;
        Ok(exists)
    }
}
