use std::str::FromStr;

use domain::entities::{
    NewVendor, SortDir, Vendor, VendorFilter, VendorSort, VendorSortField, VendorType, VendorUpdate,
};
use domain::errors::DomainError;
use domain::repositories::VendorRepository;
use sqlx::PgPool;
use sqlx::types::chrono::{DateTime, Utc};
use uuid::Uuid;

const VENDOR_COLS: &str = "id, code, name, vendor_type, tax_code, address, phone, email, notes, \
    created_at, updated_at";

/// Lọc optional bằng `($n::text IS NULL OR col = $n)` — cast để Postgres biết kiểu khi NULL.
const VENDOR_LIST_WHERE: &str = r#"
    WHERE deleted_at IS NULL
      AND ($1::text IS NULL OR code = $1)
      AND ($2::text IS NULL OR name = $2)
      AND ($3::text IS NULL OR vendor_type = $3)
"#;

/// Map lỗi DB → DomainError theo TÊN constraint/index (PG trả tên cả với unique index).
fn map_sqlx_err(e: sqlx::Error) -> DomainError {
    if let sqlx::Error::Database(db) = &e {
        match db.constraint() {
            Some("uq_vendors_code") => {
                return DomainError::AlreadyExists("Mã nhà cung cấp đã tồn tại".into());
            }
            Some("chk_vendors_type") => {
                return DomainError::Validation("Loại nhà cung cấp không hợp lệ".into());
            }
            _ => {}
        }
    }
    DomainError::Internal(e.to_string())
}

fn order_by_clause(sort: &[VendorSort]) -> String {
    let mut parts: Vec<String> = sort
        .iter()
        .map(|s| {
            let col = match s.field {
                VendorSortField::Code => "code",
                VendorSortField::Name => "name",
                VendorSortField::VendorType => "vendor_type",
                VendorSortField::CreatedAt => "created_at",
                VendorSortField::UpdatedAt => "updated_at",
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
struct VendorRow {
    id: Uuid,
    code: String,
    name: String,
    vendor_type: String,
    tax_code: Option<String>,
    address: Option<String>,
    phone: Option<String>,
    email: Option<String>,
    notes: Option<String>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl TryFrom<VendorRow> for Vendor {
    type Error = DomainError;
    fn try_from(r: VendorRow) -> Result<Self, Self::Error> {
        Ok(Vendor {
            id: r.id,
            code: r.code,
            name: r.name,
            vendor_type: VendorType::from_str(&r.vendor_type).map_err(DomainError::Internal)?,
            tax_code: r.tax_code,
            address: r.address,
            phone: r.phone,
            email: r.email,
            notes: r.notes,
            created_at: r.created_at,
            updated_at: r.updated_at,
        })
    }
}

#[derive(Clone)]
pub struct PgVendorRepository {
    pool: PgPool,
}

impl PgVendorRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

impl VendorRepository for PgVendorRepository {
    async fn create(&self, new_vendor: NewVendor) -> Result<Vendor, DomainError> {
        let sql = format!(
            r#"
            INSERT INTO vendors
                (code, name, vendor_type, tax_code, address, phone, email, notes)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            RETURNING {VENDOR_COLS}
            "#
        );
        let row: VendorRow = sqlx::query_as(&sql)
            .bind(&new_vendor.code)
            .bind(&new_vendor.name)
            .bind(new_vendor.vendor_type.as_str())
            .bind(&new_vendor.tax_code)
            .bind(&new_vendor.address)
            .bind(&new_vendor.phone)
            .bind(&new_vendor.email)
            .bind(&new_vendor.notes)
            .fetch_one(&self.pool)
            .await
            .map_err(map_sqlx_err)?; // unique_violation → "Mã nhà cung cấp đã tồn tại"
        Vendor::try_from(row)
    }

    async fn find_by_id(&self, id: Uuid) -> Result<Option<Vendor>, DomainError> {
        let sql = format!("SELECT {VENDOR_COLS} FROM vendors WHERE id = $1 AND deleted_at IS NULL");
        let row: Option<VendorRow> = sqlx::query_as(&sql)
            .bind(id)
            .fetch_optional(&self.pool)
            .await
            .map_err(map_sqlx_err)?;
        row.map(Vendor::try_from).transpose()
    }

    async fn list(&self, filter: VendorFilter) -> Result<(Vec<Vendor>, i64), DomainError> {
        let count_sql = format!("SELECT COUNT(*) FROM vendors {VENDOR_LIST_WHERE}");
        let total: i64 = sqlx::query_scalar(&count_sql)
            .bind(&filter.code)
            .bind(&filter.name)
            .bind(&filter.vendor_type)
            .fetch_one(&self.pool)
            .await
            .map_err(map_sqlx_err)?;

        let order = order_by_clause(&filter.sort);
        let list_sql = format!(
            "SELECT {VENDOR_COLS} FROM vendors {VENDOR_LIST_WHERE} {order} LIMIT $4 OFFSET $5"
        );
        let rows: Vec<VendorRow> = sqlx::query_as(&list_sql)
            .bind(&filter.code)
            .bind(&filter.name)
            .bind(&filter.vendor_type)
            .bind(filter.limit)
            .bind(filter.offset)
            .fetch_all(&self.pool)
            .await
            .map_err(map_sqlx_err)?;

        let vendors = rows
            .into_iter()
            .map(Vendor::try_from)
            .collect::<Result<Vec<_>, _>>()?;
        Ok((vendors, total))
    }

    async fn update(&self, id: Uuid, changes: VendorUpdate) -> Result<Option<Vendor>, DomainError> {
        let vendor_type = changes.vendor_type.map(|v| v.as_str());
        let sql = format!(
            r#"
            UPDATE vendors SET
                code = COALESCE($2, code),
                name = COALESCE($3, name),
                vendor_type = COALESCE($4, vendor_type),
                tax_code = COALESCE($5, tax_code),
                address = COALESCE($6, address),
                phone = COALESCE($7, phone),
                email = COALESCE($8, email),
                notes = COALESCE($9, notes)
            WHERE id = $1 AND deleted_at IS NULL
            RETURNING {VENDOR_COLS}
            "#
        );
        let row: Option<VendorRow> = sqlx::query_as(&sql)
            .bind(id)
            .bind(&changes.code)
            .bind(&changes.name)
            .bind(vendor_type)
            .bind(&changes.tax_code)
            .bind(&changes.address)
            .bind(&changes.phone)
            .bind(&changes.email)
            .bind(&changes.notes)
            .fetch_optional(&self.pool)
            .await
            .map_err(map_sqlx_err)?;
        row.map(Vendor::try_from).transpose()
    }

    async fn soft_delete(&self, id: Uuid) -> Result<(), DomainError> {
        let res =
            sqlx::query("UPDATE vendors SET deleted_at = NOW() WHERE id = $1 AND deleted_at IS NULL")
                .bind(id)
                .execute(&self.pool)
                .await
                .map_err(map_sqlx_err)?;
        if res.rows_affected() == 0 {
            return Err(DomainError::NotFound("Nhà cung cấp không tồn tại".into()));
        }
        Ok(())
    }
}
