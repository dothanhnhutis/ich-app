use std::str::FromStr;

use domain::entities::{
    NewRole, Permission, Role, RoleFilter, RoleSort, RoleSortField, RoleStatus, RoleUpdate, SortDir,
};
use application::errors::AppError;
use application::ports::RoleRepository;
use sqlx::PgPool;
use sqlx::types::chrono::{DateTime, Utc};
use uuid::Uuid;

const PERMISSION_CODES_FOR_USER: &str = r#"
    SELECT DISTINCT p.code
    FROM user_roles ur
    JOIN role_permissions rp ON rp.role_id = ur.role_id
    JOIN permissions p ON p.id = rp.permission_id
    JOIN roles r ON r.id = ur.role_id
    WHERE ur.user_id = $1
      AND r.status = 'ACTIVE'
      AND r.deleted_at IS NULL
"#;

const ALL_PERMISSIONS: &str = "SELECT id, code, description FROM permissions ORDER BY code";

const ROLE_COLS: &str =
    "id, name, description, status, deactivated_at, can_delete, can_update, created_at, updated_at";

/// Mệnh đề lọc dùng chung cho count + list. Lọc optional bằng pattern
/// `($n::text IS NULL OR col = $n)` — cast `::text` để Postgres biết kiểu khi NULL.
const ROLE_LIST_WHERE: &str = r#"
    WHERE deleted_at IS NULL
      AND ($1::text IS NULL OR name = $1)
      AND ($2::text IS NULL OR description = $2)
      AND ($3::text IS NULL OR status = $3)
"#;

fn map_sqlx_err(e: sqlx::Error) -> AppError {
    if let sqlx::Error::Database(db) = &e
        && db.is_foreign_key_violation()
    {
        return AppError::Validation("Một số quyền không tồn tại".into());
    }
    AppError::Internal(e.to_string())
}

/// Dựng mệnh đề ORDER BY từ danh sách sort (cột + hướng đều là literal từ match → an toàn injection).
/// Luôn nối tiebreaker `id DESC` để phân trang ổn định; rỗng → mặc định created_at DESC.
fn order_by_clause(sort: &[RoleSort]) -> String {
    let mut parts: Vec<String> = sort
        .iter()
        .map(|s| {
            let col = match s.field {
                RoleSortField::Name => "name",
                RoleSortField::Status => "status",
                RoleSortField::CreatedAt => "created_at",
                RoleSortField::UpdatedAt => "updated_at",
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
struct PermissionRow {
    id: Uuid,
    code: String,
    description: String,
}

impl From<PermissionRow> for Permission {
    fn from(r: PermissionRow) -> Self {
        Self {
            id: r.id,
            code: r.code,
            description: r.description,
        }
    }
}

#[derive(sqlx::FromRow)]
struct RoleRow {
    id: Uuid,
    name: String,
    description: String,
    status: String,
    deactivated_at: Option<DateTime<Utc>>,
    can_delete: bool,
    can_update: bool,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl TryFrom<RoleRow> for Role {
    type Error = AppError;
    fn try_from(r: RoleRow) -> Result<Self, Self::Error> {
        Ok(Role {
            id: r.id,
            name: r.name,
            description: r.description,
            status: RoleStatus::from_str(&r.status).map_err(AppError::Internal)?,
            deactivated_at: r.deactivated_at,
            can_delete: r.can_delete,
            can_update: r.can_update,
            created_at: r.created_at,
            updated_at: r.updated_at,
        })
    }
}

#[derive(Clone)]
pub struct PgRoleRepository {
    pool: PgPool,
}

impl PgRoleRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

impl RoleRepository for PgRoleRepository {
    async fn find_permission_codes_for_user(
        &self,
        user_id: Uuid,
    ) -> Result<Vec<String>, AppError> {
        let codes: Vec<String> = sqlx::query_scalar(PERMISSION_CODES_FOR_USER)
            .bind(user_id)
            .fetch_all(&self.pool)
            .await
            .map_err(map_sqlx_err)?;
        Ok(codes)
    }

    async fn find_all_permissions(&self) -> Result<Vec<Permission>, AppError> {
        let rows: Vec<PermissionRow> = sqlx::query_as(ALL_PERMISSIONS)
            .fetch_all(&self.pool)
            .await
            .map_err(map_sqlx_err)?;
        Ok(rows.into_iter().map(Permission::from).collect())
    }

    async fn create_with_permissions(
        &self,
        new_role: NewRole,
        permission_ids: &[Uuid],
    ) -> Result<Role, AppError> {
        let mut tx = self.pool.begin().await.map_err(map_sqlx_err)?;

        let sql = format!(
            "INSERT INTO roles (name, description) VALUES ($1, $2) RETURNING {ROLE_COLS}"
        );
        let row: RoleRow = sqlx::query_as(&sql)
            .bind(&new_role.name)
            .bind(&new_role.description)
            .fetch_one(&mut *tx)
            .await
            .map_err(map_sqlx_err)?;

        // Gán permission — FK RESTRICT đảm bảo permission tồn tại
        // (fk_violation → "Một số quyền không tồn tại").
        for permission_id in permission_ids {
            sqlx::query("INSERT INTO role_permissions (role_id, permission_id) VALUES ($1, $2)")
                .bind(row.id)
                .bind(permission_id)
                .execute(&mut *tx)
                .await
                .map_err(map_sqlx_err)?;
        }

        tx.commit().await.map_err(map_sqlx_err)?;
        Role::try_from(row)
    }

    async fn find_by_id(&self, id: Uuid) -> Result<Option<Role>, AppError> {
        let sql = format!("SELECT {ROLE_COLS} FROM roles WHERE id = $1 AND deleted_at IS NULL");
        let row: Option<RoleRow> = sqlx::query_as(&sql)
            .bind(id)
            .fetch_optional(&self.pool)
            .await
            .map_err(map_sqlx_err)?;
        row.map(Role::try_from).transpose()
    }

    async fn find_permissions_for_role(&self, role_id: Uuid) -> Result<Vec<Permission>, AppError> {
        let rows: Vec<PermissionRow> = sqlx::query_as(
            r#"
            SELECT p.id, p.code, p.description
            FROM role_permissions rp
            JOIN permissions p ON p.id = rp.permission_id
            WHERE rp.role_id = $1
            ORDER BY p.code
            "#,
        )
        .bind(role_id)
        .fetch_all(&self.pool)
        .await
        .map_err(map_sqlx_err)?;
        Ok(rows.into_iter().map(Permission::from).collect())
    }

    async fn find_roles_for_user(&self, user_id: Uuid) -> Result<Vec<Role>, AppError> {
        // Qualify `r.` vì cột created_at trùng ở cả user_roles lẫn roles.
        let rows: Vec<RoleRow> = sqlx::query_as(
            r#"
            SELECT r.id, r.name, r.description, r.status, r.deactivated_at,
                   r.can_delete, r.can_update, r.created_at, r.updated_at
            FROM user_roles ur
            JOIN roles r ON r.id = ur.role_id
            WHERE ur.user_id = $1 AND r.deleted_at IS NULL
            ORDER BY r.created_at DESC
            "#,
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await
        .map_err(map_sqlx_err)?;
        rows.into_iter()
            .map(Role::try_from)
            .collect::<Result<Vec<_>, _>>()
    }

    async fn list(&self, filter: RoleFilter) -> Result<(Vec<Role>, i64), AppError> {
        let status = filter.status.map(|s| s.as_str());

        let count_sql = format!("SELECT COUNT(*) FROM roles {ROLE_LIST_WHERE}");
        let total: i64 = sqlx::query_scalar(&count_sql)
            .bind(&filter.name)
            .bind(&filter.description)
            .bind(status)
            .fetch_one(&self.pool)
            .await
            .map_err(map_sqlx_err)?;

        let order = order_by_clause(&filter.sort);
        let list_sql = format!(
            "SELECT {ROLE_COLS} FROM roles {ROLE_LIST_WHERE} {order} LIMIT $4 OFFSET $5"
        );
        let rows: Vec<RoleRow> = sqlx::query_as(&list_sql)
            .bind(&filter.name)
            .bind(&filter.description)
            .bind(status)
            .bind(filter.limit)
            .bind(filter.offset)
            .fetch_all(&self.pool)
            .await
            .map_err(map_sqlx_err)?;

        let roles = rows
            .into_iter()
            .map(Role::try_from)
            .collect::<Result<Vec<_>, _>>()?;
        Ok((roles, total))
    }

    async fn update(&self, id: Uuid, changes: RoleUpdate) -> Result<Option<Role>, AppError> {
        let status = changes.status.map(|s| s.as_str());
        let mut tx = self.pool.begin().await.map_err(map_sqlx_err)?;

        let sql = format!(
            r#"
            UPDATE roles SET
                name = COALESCE($2, name),
                description = COALESCE($3, description),
                status = COALESCE($4, status),
                deactivated_at = CASE
                    WHEN $4 = 'DEACTIVATED' THEN NOW()
                    WHEN $4 = 'ACTIVE' THEN NULL
                    ELSE deactivated_at
                END
            WHERE id = $1 AND deleted_at IS NULL
            RETURNING {ROLE_COLS}
            "#
        );
        let row: Option<RoleRow> = sqlx::query_as(&sql)
            .bind(id)
            .bind(&changes.name)
            .bind(&changes.description)
            .bind(status)
            .fetch_optional(&mut *tx)
            .await
            .map_err(map_sqlx_err)?;

        let Some(row) = row else {
            return Ok(None); // role không tồn tại / đã xoá mềm — drop tx = rollback
        };

        // Thay thế toàn bộ tập permission (chỉ khi client gửi permission_ids).
        if let Some(permission_ids) = &changes.permission_ids {
            sqlx::query("DELETE FROM role_permissions WHERE role_id = $1")
                .bind(id)
                .execute(&mut *tx)
                .await
                .map_err(map_sqlx_err)?;
            for permission_id in permission_ids {
                sqlx::query("INSERT INTO role_permissions (role_id, permission_id) VALUES ($1, $2)")
                    .bind(id)
                    .bind(permission_id)
                    .execute(&mut *tx)
                    .await
                    .map_err(map_sqlx_err)?; // fk_violation → "Một số quyền không tồn tại"
            }
        }

        tx.commit().await.map_err(map_sqlx_err)?;
        Role::try_from(row).map(Some)
    }

    async fn soft_delete(&self, id: Uuid) -> Result<(), AppError> {
        let res = sqlx::query("UPDATE roles SET deleted_at = NOW() WHERE id = $1 AND deleted_at IS NULL")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(map_sqlx_err)?;
        if res.rows_affected() == 0 {
            return Err(AppError::NotFound("Vai trò không tồn tại".into()));
        }
        Ok(())
    }
}
