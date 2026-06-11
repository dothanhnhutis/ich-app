use std::str::FromStr;

use domain::entities::{
    Bom, BomFilter, BomLine, BomLineType, BomLineUpdate, BomSort, BomSortField, BomStatus, BomType,
    BomUpdate, NewBom, NewBomLine, QtyBasis, SortDir,
};
use domain::errors::DomainError;
use domain::repositories::BomRepository;
use sqlx::PgPool;
use sqlx::types::chrono::{DateTime, Utc};
use uuid::Uuid;

/// Cột số = DOUBLE PRECISION (f64 native, không cần cast).
const BOM_COLS: &str = "id, output_item_id, bom_type, code, name, version_no, status, is_default, \
    qty_basis, output_qty, output_uom, effective_from, effective_to, notes, \
    created_at, updated_at";

const BOM_LINE_COLS: &str = "id, bom_id, component_item_id, line_no, line_type, \
    quantity, input_uom, input_qty, scrap_pct, is_gift, notes, created_at, updated_at";

const BOM_LIST_WHERE: &str = r#"
    WHERE deleted_at IS NULL
      AND ($1::uuid IS NULL OR output_item_id = $1)
      AND ($2::text IS NULL OR bom_type = $2)
      AND ($3::text IS NULL OR status = $3)
      AND ($4::text IS NULL OR code = $4)
"#;

fn map_sqlx_err(e: sqlx::Error) -> DomainError {
    if let sqlx::Error::Database(db) = &e {
        match db.constraint() {
            Some("uq_boms_code") => {
                return DomainError::AlreadyExists("Mã BOM đã tồn tại".into());
            }
            Some("uq_boms_item_type_ver") => {
                return DomainError::AlreadyExists(
                    "Phiên bản BOM cho item này (theo loại) đã tồn tại".into(),
                );
            }
            Some("uq_boms_default_active") => {
                return DomainError::Validation(
                    "Item đã có một BOM mặc định đang ACTIVE cùng loại".into(),
                );
            }
            Some("chk_boms_type") => {
                return DomainError::Validation("Loại BOM không hợp lệ".into());
            }
            Some("chk_boms_status") => {
                return DomainError::Validation("Trạng thái BOM không hợp lệ".into());
            }
            Some("chk_boms_qty_basis") => {
                return DomainError::Validation("Cơ sở định lượng không hợp lệ".into());
            }
            Some("chk_boms_out_qty") => {
                return DomainError::Validation("Sản lượng đầu ra phải lớn hơn 0".into());
            }
            Some("chk_boms_effective") => {
                return DomainError::Validation(
                    "Thời điểm hiệu lực 'đến' phải sau 'từ'".into(),
                );
            }
            Some("fk_boms_output_item") => {
                return DomainError::Validation("Item đầu ra không tồn tại".into());
            }
            Some("uq_bom_lines_line_no") => {
                return DomainError::AlreadyExists("Số dòng đã tồn tại trong BOM".into());
            }
            Some("uq_bom_lines_component") => {
                return DomainError::AlreadyExists("Thành phần đã có trong BOM".into());
            }
            Some("chk_bom_lines_type") => {
                return DomainError::Validation("Loại dòng BOM không hợp lệ".into());
            }
            Some("chk_bom_lines_qty") => {
                return DomainError::Validation("Số lượng phải lớn hơn 0".into());
            }
            Some("chk_bom_lines_input_qty") => {
                return DomainError::Validation("Số lượng nhập phải lớn hơn 0".into());
            }
            Some("chk_bom_lines_scrap") => {
                return DomainError::Validation("Hao hụt phải trong khoảng 0 đến dưới 100".into());
            }
            Some("fk_bom_lines_bom") => {
                return DomainError::Validation("BOM không tồn tại".into());
            }
            Some("fk_bom_lines_component") => {
                return DomainError::Validation("Thành phần (item) không tồn tại".into());
            }
            _ => {}
        }
    }
    DomainError::Internal(e.to_string())
}

fn order_by_clause(sort: &[BomSort]) -> String {
    let mut parts: Vec<String> = sort
        .iter()
        .map(|s| {
            let col = match s.field {
                BomSortField::Code => "code",
                BomSortField::Name => "name",
                BomSortField::Status => "status",
                BomSortField::VersionNo => "version_no",
                BomSortField::CreatedAt => "created_at",
                BomSortField::UpdatedAt => "updated_at",
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
struct BomRow {
    id: Uuid,
    output_item_id: Uuid,
    bom_type: String,
    code: String,
    name: String,
    version_no: i32,
    status: String,
    is_default: bool,
    qty_basis: String,
    output_qty: f64,
    output_uom: String,
    effective_from: Option<DateTime<Utc>>,
    effective_to: Option<DateTime<Utc>>,
    notes: Option<String>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl TryFrom<BomRow> for Bom {
    type Error = DomainError;
    fn try_from(r: BomRow) -> Result<Self, Self::Error> {
        Ok(Bom {
            id: r.id,
            output_item_id: r.output_item_id,
            bom_type: BomType::from_str(&r.bom_type).map_err(DomainError::Internal)?,
            code: r.code,
            name: r.name,
            version_no: r.version_no,
            status: BomStatus::from_str(&r.status).map_err(DomainError::Internal)?,
            is_default: r.is_default,
            qty_basis: QtyBasis::from_str(&r.qty_basis).map_err(DomainError::Internal)?,
            output_qty: r.output_qty,
            output_uom: r.output_uom,
            effective_from: r.effective_from,
            effective_to: r.effective_to,
            notes: r.notes,
            created_at: r.created_at,
            updated_at: r.updated_at,
        })
    }
}

#[derive(sqlx::FromRow)]
struct BomLineRow {
    id: Uuid,
    bom_id: Uuid,
    component_item_id: Uuid,
    line_no: i32,
    line_type: String,
    quantity: f64,
    input_uom: Option<String>,
    input_qty: Option<f64>,
    scrap_pct: f64,
    is_gift: bool,
    notes: Option<String>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl TryFrom<BomLineRow> for BomLine {
    type Error = DomainError;
    fn try_from(r: BomLineRow) -> Result<Self, Self::Error> {
        Ok(BomLine {
            id: r.id,
            bom_id: r.bom_id,
            component_item_id: r.component_item_id,
            line_no: r.line_no,
            line_type: BomLineType::from_str(&r.line_type).map_err(DomainError::Internal)?,
            quantity: r.quantity,
            input_uom: r.input_uom,
            input_qty: r.input_qty,
            scrap_pct: r.scrap_pct,
            is_gift: r.is_gift,
            notes: r.notes,
            created_at: r.created_at,
            updated_at: r.updated_at,
        })
    }
}

#[derive(Clone)]
pub struct PgBomRepository {
    pool: PgPool,
}

impl PgBomRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

const INSERT_LINE_SQL: &str = r#"
    INSERT INTO bom_lines
        (bom_id, component_item_id, line_no, line_type, quantity, input_uom, input_qty,
         scrap_pct, is_gift, notes)
    VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
"#;

impl BomRepository for PgBomRepository {
    async fn create_with_lines(
        &self,
        new_bom: NewBom,
        lines: &[NewBomLine],
    ) -> Result<(Bom, Vec<BomLine>), DomainError> {
        let mut tx = self.pool.begin().await.map_err(map_sqlx_err)?;

        let bom_sql = format!(
            r#"
            INSERT INTO boms
                (output_item_id, bom_type, code, name, version_no, status, is_default,
                 qty_basis, output_qty, output_uom, effective_from, effective_to, notes)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)
            RETURNING {BOM_COLS}
            "#
        );
        let bom_row: BomRow = sqlx::query_as(&bom_sql)
            .bind(new_bom.output_item_id)
            .bind(new_bom.bom_type.as_str())
            .bind(&new_bom.code)
            .bind(&new_bom.name)
            .bind(new_bom.version_no)
            .bind(new_bom.status.as_str())
            .bind(new_bom.is_default)
            .bind(new_bom.qty_basis.as_str())
            .bind(new_bom.output_qty)
            .bind(&new_bom.output_uom)
            .bind(new_bom.effective_from)
            .bind(new_bom.effective_to)
            .bind(&new_bom.notes)
            .fetch_one(&mut *tx)
            .await
            .map_err(map_sqlx_err)?;

        let bom_id = bom_row.id;
        let line_sql = format!("{INSERT_LINE_SQL} RETURNING {BOM_LINE_COLS}");
        let mut line_rows: Vec<BomLineRow> = Vec::with_capacity(lines.len());
        for l in lines {
            let row: BomLineRow = sqlx::query_as(&line_sql)
                .bind(bom_id)
                .bind(l.component_item_id)
                .bind(l.line_no)
                .bind(l.line_type.as_str())
                .bind(l.quantity)
                .bind(&l.input_uom)
                .bind(l.input_qty)
                .bind(l.scrap_pct)
                .bind(l.is_gift)
                .bind(&l.notes)
                .fetch_one(&mut *tx)
                .await
                .map_err(map_sqlx_err)?;
            line_rows.push(row);
        }

        tx.commit().await.map_err(map_sqlx_err)?;

        let bom = Bom::try_from(bom_row)?;
        let lines = line_rows
            .into_iter()
            .map(BomLine::try_from)
            .collect::<Result<Vec<_>, _>>()?;
        Ok((bom, lines))
    }

    async fn find_by_id(&self, id: Uuid) -> Result<Option<Bom>, DomainError> {
        let sql = format!("SELECT {BOM_COLS} FROM boms WHERE id = $1 AND deleted_at IS NULL");
        let row: Option<BomRow> = sqlx::query_as(&sql)
            .bind(id)
            .fetch_optional(&self.pool)
            .await
            .map_err(map_sqlx_err)?;
        row.map(Bom::try_from).transpose()
    }

    async fn find_with_lines(
        &self,
        id: Uuid,
    ) -> Result<Option<(Bom, Vec<BomLine>)>, DomainError> {
        let Some(bom) = self.find_by_id(id).await? else {
            return Ok(None);
        };
        let lines = self.list_lines(id).await?;
        Ok(Some((bom, lines)))
    }

    async fn list(&self, filter: BomFilter) -> Result<(Vec<Bom>, i64), DomainError> {
        let count_sql = format!("SELECT COUNT(*) FROM boms {BOM_LIST_WHERE}");
        let total: i64 = sqlx::query_scalar(&count_sql)
            .bind(filter.output_item_id)
            .bind(&filter.bom_type)
            .bind(&filter.status)
            .bind(&filter.code)
            .fetch_one(&self.pool)
            .await
            .map_err(map_sqlx_err)?;

        let order = order_by_clause(&filter.sort);
        let list_sql =
            format!("SELECT {BOM_COLS} FROM boms {BOM_LIST_WHERE} {order} LIMIT $5 OFFSET $6");
        let rows: Vec<BomRow> = sqlx::query_as(&list_sql)
            .bind(filter.output_item_id)
            .bind(&filter.bom_type)
            .bind(&filter.status)
            .bind(&filter.code)
            .bind(filter.limit)
            .bind(filter.offset)
            .fetch_all(&self.pool)
            .await
            .map_err(map_sqlx_err)?;

        let boms = rows
            .into_iter()
            .map(Bom::try_from)
            .collect::<Result<Vec<_>, _>>()?;
        Ok((boms, total))
    }

    async fn update(&self, id: Uuid, changes: BomUpdate) -> Result<Option<Bom>, DomainError> {
        let status = changes.status.map(|s| s.as_str());
        let qty_basis = changes.qty_basis.map(|q| q.as_str());
        let sql = format!(
            r#"
            UPDATE boms SET
                code = COALESCE($2, code),
                name = COALESCE($3, name),
                version_no = COALESCE($4, version_no),
                status = COALESCE($5, status),
                is_default = COALESCE($6, is_default),
                qty_basis = COALESCE($7, qty_basis),
                output_qty = COALESCE($8, output_qty),
                output_uom = COALESCE($9, output_uom),
                effective_from = COALESCE($10, effective_from),
                effective_to = COALESCE($11, effective_to),
                notes = COALESCE($12, notes)
            WHERE id = $1 AND deleted_at IS NULL
            RETURNING {BOM_COLS}
            "#
        );
        let row: Option<BomRow> = sqlx::query_as(&sql)
            .bind(id)
            .bind(&changes.code)
            .bind(&changes.name)
            .bind(changes.version_no)
            .bind(status)
            .bind(changes.is_default)
            .bind(qty_basis)
            .bind(changes.output_qty)
            .bind(&changes.output_uom)
            .bind(changes.effective_from)
            .bind(changes.effective_to)
            .bind(&changes.notes)
            .fetch_optional(&self.pool)
            .await
            .map_err(map_sqlx_err)?;
        row.map(Bom::try_from).transpose()
    }

    async fn soft_delete(&self, id: Uuid) -> Result<(), DomainError> {
        let mut tx = self.pool.begin().await.map_err(map_sqlx_err)?;
        let res =
            sqlx::query("UPDATE boms SET deleted_at = NOW() WHERE id = $1 AND deleted_at IS NULL")
                .bind(id)
                .execute(&mut *tx)
                .await
                .map_err(map_sqlx_err)?;
        if res.rows_affected() == 0 {
            return Err(DomainError::NotFound("BOM không tồn tại".into())); // drop tx = rollback
        }
        // Cascade xoá mềm các dòng (Hybrid: BOM sở hữu lines).
        sqlx::query("UPDATE bom_lines SET deleted_at = NOW() WHERE bom_id = $1 AND deleted_at IS NULL")
            .bind(id)
            .execute(&mut *tx)
            .await
            .map_err(map_sqlx_err)?;
        tx.commit().await.map_err(map_sqlx_err)?;
        Ok(())
    }

    async fn add_line(&self, bom_id: Uuid, l: NewBomLine) -> Result<BomLine, DomainError> {
        let sql = format!("{INSERT_LINE_SQL} RETURNING {BOM_LINE_COLS}");
        let row: BomLineRow = sqlx::query_as(&sql)
            .bind(bom_id)
            .bind(l.component_item_id)
            .bind(l.line_no)
            .bind(l.line_type.as_str())
            .bind(l.quantity)
            .bind(&l.input_uom)
            .bind(l.input_qty)
            .bind(l.scrap_pct)
            .bind(l.is_gift)
            .bind(&l.notes)
            .fetch_one(&self.pool)
            .await
            .map_err(map_sqlx_err)?;
        BomLine::try_from(row)
    }

    async fn update_line(
        &self,
        bom_id: Uuid,
        line_id: Uuid,
        changes: BomLineUpdate,
    ) -> Result<Option<BomLine>, DomainError> {
        let line_type = changes.line_type.map(|t| t.as_str());
        let sql = format!(
            r#"
            UPDATE bom_lines SET
                line_no = COALESCE($3, line_no),
                line_type = COALESCE($4, line_type),
                quantity = COALESCE($5, quantity),
                input_uom = COALESCE($6, input_uom),
                input_qty = COALESCE($7, input_qty),
                scrap_pct = COALESCE($8, scrap_pct),
                is_gift = COALESCE($9, is_gift),
                notes = COALESCE($10, notes)
            WHERE id = $2 AND bom_id = $1 AND deleted_at IS NULL
            RETURNING {BOM_LINE_COLS}
            "#
        );
        let row: Option<BomLineRow> = sqlx::query_as(&sql)
            .bind(bom_id)
            .bind(line_id)
            .bind(changes.line_no)
            .bind(line_type)
            .bind(changes.quantity)
            .bind(&changes.input_uom)
            .bind(changes.input_qty)
            .bind(changes.scrap_pct)
            .bind(changes.is_gift)
            .bind(&changes.notes)
            .fetch_optional(&self.pool)
            .await
            .map_err(map_sqlx_err)?;
        row.map(BomLine::try_from).transpose()
    }

    async fn soft_delete_line(&self, bom_id: Uuid, line_id: Uuid) -> Result<(), DomainError> {
        let res = sqlx::query(
            "UPDATE bom_lines SET deleted_at = NOW() WHERE id = $1 AND bom_id = $2 AND deleted_at IS NULL",
        )
        .bind(line_id)
        .bind(bom_id)
        .execute(&self.pool)
        .await
        .map_err(map_sqlx_err)?;
        if res.rows_affected() == 0 {
            return Err(DomainError::NotFound("Dòng BOM không tồn tại".into()));
        }
        Ok(())
    }

    async fn list_lines(&self, bom_id: Uuid) -> Result<Vec<BomLine>, DomainError> {
        let sql = format!(
            "SELECT {BOM_LINE_COLS} FROM bom_lines \
             WHERE bom_id = $1 AND deleted_at IS NULL ORDER BY line_no ASC, id ASC"
        );
        let rows: Vec<BomLineRow> = sqlx::query_as(&sql)
            .bind(bom_id)
            .fetch_all(&self.pool)
            .await
            .map_err(map_sqlx_err)?;
        rows.into_iter().map(BomLine::try_from).collect()
    }

    async fn would_create_cycle(
        &self,
        component_item_id: Uuid,
        output_item_id: Uuid,
    ) -> Result<bool, DomainError> {
        // Duyệt đệ quy các thành phần mà `component` phụ thuộc (trực/gián tiếp);
        // nếu `output` xuất hiện → thêm dòng này tạo chu trình.
        let cycles: bool = sqlx::query_scalar(
            r#"
            WITH RECURSIVE deps AS (
                SELECT bl.component_item_id AS item_id
                FROM boms b
                JOIN bom_lines bl ON bl.bom_id = b.id
                WHERE b.output_item_id = $1
                  AND b.deleted_at IS NULL AND bl.deleted_at IS NULL
                UNION
                SELECT bl.component_item_id
                FROM deps d
                JOIN boms b ON b.output_item_id = d.item_id AND b.deleted_at IS NULL
                JOIN bom_lines bl ON bl.bom_id = b.id AND bl.deleted_at IS NULL
            )
            SELECT EXISTS(SELECT 1 FROM deps WHERE item_id = $2)
            "#,
        )
        .bind(component_item_id)
        .bind(output_item_id)
        .fetch_one(&self.pool)
        .await
        .map_err(map_sqlx_err)?;
        Ok(cycles)
    }
}
