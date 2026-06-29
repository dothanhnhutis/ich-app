use std::str::FromStr;

use domain::entities::{
    NewUser, SortDir, User, UserFilter, UserSort, UserSortField, UserStatus, UserUpdate,
};
use application::errors::AppError;
use application::ports::UserRepository;
use sqlx::PgPool;
use sqlx::types::chrono::{DateTime, Utc};
use uuid::Uuid;

/// Struct riêng cho DB layer — tách biệt khỏi domain entity
#[derive(Debug, sqlx::FromRow)]
struct UserRow {
    id: Uuid,
    email: String,
    password_hash: Option<String>,
    username: Option<String>,
    status: String,
    deactivated_at: Option<DateTime<Utc>>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

/// Mapping từ DB row → Domain entity
impl TryFrom<UserRow> for User {
    type Error = AppError;
    fn try_from(row: UserRow) -> Result<Self, Self::Error> {
        Ok(User {
            id: row.id,
            email: row.email,
            password_hash: row.password_hash,
            username: row.username,
            status: UserStatus::from_str(&row.status).map_err(AppError::Internal)?,
            deactivated_at: row.deactivated_at,
            created_at: row.created_at,
            updated_at: row.updated_at,
        })
    }
}

const SELECT_USER: &str = r#"
    SELECT id, email, password_hash, username, status,
           deactivated_at, created_at, updated_at
    FROM users
"#;

/// Mệnh đề lọc dùng chung cho count + list (bỏ user đã xoá mềm).
const USER_LIST_WHERE: &str = r#"
    WHERE deleted_at IS NULL
      AND ($1::text IS NULL OR email = $1)
      AND ($2::text IS NULL OR username = $2)
      AND ($3::text IS NULL OR status = $3)
"#;

/// Dựng ORDER BY từ sort (cột + hướng đều literal từ match → an toàn injection).
/// Luôn nối tiebreaker `id DESC`; rỗng → mặc định created_at DESC.
fn user_order_by_clause(sort: &[UserSort]) -> String {
    let mut parts: Vec<String> = sort
        .iter()
        .map(|s| {
            let col = match s.field {
                UserSortField::Email => "email",
                UserSortField::Username => "username",
                UserSortField::Status => "status",
                UserSortField::CreatedAt => "created_at",
                UserSortField::UpdatedAt => "updated_at",
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

fn map_sqlx_err(e: sqlx::Error) -> AppError {
    if let sqlx::Error::Database(db) = &e {
        if db.is_unique_violation() {
            return AppError::Validation("Email đã tồn tại".into());
        }
        if db.is_foreign_key_violation() {
            return AppError::Validation("Role không tồn tại".into());
        }
    }
    AppError::Internal(e.to_string())
}

#[derive(Clone)]
pub struct PgUserRepository {
    pool: PgPool,
}

impl PgUserRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

impl UserRepository for PgUserRepository {
    async fn find_by_email(&self, email: &str) -> Result<Option<User>, AppError> {
        let sql = format!("{} WHERE email = $1 AND deleted_at IS NULL", SELECT_USER);
        let row: Option<UserRow> = sqlx::query_as(&sql)
            .bind(email)
            .fetch_optional(&self.pool)
            .await
            .map_err(map_sqlx_err)?;

        row.map(User::try_from).transpose()
    }

    async fn find_by_id(&self, id: uuid::Uuid) -> Result<Option<User>, AppError> {
        let sql = format!("{} WHERE id = $1 AND deleted_at IS NULL", SELECT_USER);
        let row: Option<UserRow> = sqlx::query_as(&sql)
            .bind(id)
            .fetch_optional(&self.pool)
            .await
            .map_err(map_sqlx_err)?;

        row.map(User::try_from).transpose()
    }

    async fn list(&self, filter: UserFilter) -> Result<(Vec<User>, i64), AppError> {
        let status = filter.status.map(|s| s.as_str());

        let count_sql = format!("SELECT COUNT(*) FROM users {USER_LIST_WHERE}");
        let total: i64 = sqlx::query_scalar(&count_sql)
            .bind(&filter.email)
            .bind(&filter.username)
            .bind(status)
            .fetch_one(&self.pool)
            .await
            .map_err(map_sqlx_err)?;

        let order = user_order_by_clause(&filter.sort);
        let list_sql = format!("{SELECT_USER} {USER_LIST_WHERE} {order} LIMIT $4 OFFSET $5");
        let rows: Vec<UserRow> = sqlx::query_as(&list_sql)
            .bind(&filter.email)
            .bind(&filter.username)
            .bind(status)
            .bind(filter.limit)
            .bind(filter.offset)
            .fetch_all(&self.pool)
            .await
            .map_err(map_sqlx_err)?;

        let users = rows
            .into_iter()
            .map(User::try_from)
            .collect::<Result<Vec<_>, _>>()?;
        Ok((users, total))
    }

    async fn update(&self, id: Uuid, changes: UserUpdate) -> Result<Option<User>, AppError> {
        let status = changes.status.map(|s| s.as_str());
        let sql = r#"
            UPDATE users SET
                username = COALESCE($2, username),
                status = COALESCE($3, status),
                deactivated_at = CASE
                    WHEN $3 = 'DEACTIVATED' THEN NOW()
                    WHEN $3 = 'ACTIVE' THEN NULL
                    ELSE deactivated_at
                END
            WHERE id = $1 AND deleted_at IS NULL
            RETURNING id, email, password_hash, username, status,
                      deactivated_at, created_at, updated_at
        "#;
        let row: Option<UserRow> = sqlx::query_as(sql)
            .bind(id)
            .bind(&changes.username)
            .bind(status)
            .fetch_optional(&self.pool)
            .await
            .map_err(map_sqlx_err)?;
        row.map(User::try_from).transpose()
    }

    async fn soft_delete(&self, id: Uuid) -> Result<(), AppError> {
        let res = sqlx::query("UPDATE users SET deleted_at = NOW() WHERE id = $1 AND deleted_at IS NULL")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(map_sqlx_err)?;
        if res.rows_affected() == 0 {
            return Err(AppError::NotFound("Người dùng không tồn tại".into()));
        }
        Ok(())
    }

    async fn create_with_roles(
        &self,
        new_user: NewUser,
        role_ids: &[Uuid],
    ) -> Result<User, AppError> {
        let mut tx = self.pool.begin().await.map_err(map_sqlx_err)?;

        // Tạo user — status mặc định PENDING_PASSWORD, id do DB sinh.
        let row: UserRow = sqlx::query_as(
            r#"
            INSERT INTO users (email)
            VALUES ($1)
            RETURNING id, email, password_hash, username, status,
                      deactivated_at, created_at, updated_at
            "#,
        )
        .bind(&new_user.email)
        .fetch_one(&mut *tx)
        .await
        .map_err(map_sqlx_err)?; // unique_violation → "Email đã tồn tại"

        // Gán role — FK RESTRICT đảm bảo role tồn tại (fk_violation → "Role không tồn tại").
        for role_id in role_ids {
            sqlx::query(r#"INSERT INTO user_roles (user_id, role_id) VALUES ($1, $2)"#)
                .bind(row.id)
                .bind(role_id)
                .execute(&mut *tx)
                .await
                .map_err(map_sqlx_err)?;
        }

        tx.commit().await.map_err(map_sqlx_err)?;
        User::try_from(row)
    }

    async fn activate_account(
        &self,
        user_id: Uuid,
        username: &str,
        password_hash: &str,
        token_id: Uuid,
    ) -> Result<(), AppError> {
        let mut tx = self.pool.begin().await.map_err(map_sqlx_err)?;

        sqlx::query(
            r#"
            UPDATE users
            SET username = $2, password_hash = $3, status = 'ACTIVE', password_changed_at = NOW()
            WHERE id = $1
            "#,
        )
        .bind(user_id)
        .bind(username)
        .bind(password_hash)
        .execute(&mut *tx)
        .await
        .map_err(map_sqlx_err)?;

        mark_token_used(&mut tx, token_id).await?;

        tx.commit().await.map_err(map_sqlx_err)?;
        Ok(())
    }

    async fn reset_password(
        &self,
        user_id: Uuid,
        password_hash: &str,
        token_id: Uuid,
    ) -> Result<(), AppError> {
        let mut tx = self.pool.begin().await.map_err(map_sqlx_err)?;

        sqlx::query(
            r#"
            UPDATE users
            SET password_hash = $2, password_changed_at = NOW()
            WHERE id = $1
            "#,
        )
        .bind(user_id)
        .bind(password_hash)
        .execute(&mut *tx)
        .await
        .map_err(map_sqlx_err)?;

        mark_token_used(&mut tx, token_id).await?;

        tx.commit().await.map_err(map_sqlx_err)?;
        Ok(())
    }
}

/// Đánh dấu password token đã dùng; rows_affected != 1 nghĩa là token đã bị dùng (race) → lỗi → rollback.
async fn mark_token_used(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    token_id: Uuid,
) -> Result<(), AppError> {
    let used =
        sqlx::query(r#"UPDATE password_tokens SET used_at = NOW() WHERE id = $1 AND used_at IS NULL"#)
            .bind(token_id)
            .execute(&mut **tx)
            .await
            .map_err(map_sqlx_err)?;

    if used.rows_affected() != 1 {
        return Err(AppError::Validation("Liên kết đã được sử dụng".into()));
    }
    Ok(())
}
