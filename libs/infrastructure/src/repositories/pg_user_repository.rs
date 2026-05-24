use std::str::FromStr;

use domain::entities::{User, UserStatus};
use domain::errors::DomainError;
use domain::repositories::UserRepository;
use sqlx::PgPool;
use sqlx::types::chrono::{DateTime, Utc};
use uuid::Uuid;

/// Struct riêng cho DB layer — tách biệt khỏi domain entity
#[derive(Debug, sqlx::FromRow)]
struct UserRow {
    id: Uuid,
    email: String,
    password_hash: Option<String>,
    username: String,
    status: String,
    deactivated_at: Option<DateTime<Utc>>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

/// Mapping từ DB row → Domain entity
impl TryFrom<UserRow> for User {
    type Error = DomainError;
    fn try_from(row: UserRow) -> Result<Self, Self::Error> {
        Ok(User {
            id: row.id,
            email: row.email,
            password_hash: row.password_hash,
            username: row.username,
            status: UserStatus::from_str(&row.status).map_err(DomainError::Internal)?,
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

fn map_sqlx_err(e: sqlx::Error) -> DomainError {
    if let sqlx::Error::Database(db) = &e {
        if db.is_unique_violation() {
            return DomainError::AlreadyExists("Email đã tồn tại".into());
        }
        if db.is_foreign_key_violation() {
            return DomainError::Validation("Role không tồn tại".into());
        }
    }
    DomainError::Internal(e.to_string())
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
    async fn find_by_email(&self, email: &str) -> Result<Option<User>, DomainError> {
        let sql = format!("{} WHERE email = $1", SELECT_USER);
        let row: Option<UserRow> = sqlx::query_as(&sql)
            .bind(email)
            .fetch_optional(&self.pool)
            .await
            .map_err(map_sqlx_err)?;

        row.map(User::try_from).transpose()
    }

    async fn find_by_id(&self, id: &str) -> Result<Option<User>, DomainError> {
        let uuid = uuid::Uuid::parse_str(id).map_err(|e| DomainError::Internal(e.to_string()))?;

        let row = sqlx::query_as!(
            UserRow,
            r#"
            SELECT id, email, username, password_hash
            FROM users
            WHERE id = $1
            "#,
            uuid
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DomainError::Internal(e.to_string()))?;

        Ok(row.map(|r| r.into()))
    }

    async fn create(&self, user: &User) -> Result<User, DomainError> {
        let row = sqlx::query_as!(
            UserRow,
            r#"
            INSERT INTO users (id, email, username, password_hash)
            VALUES ($1::UUID, $2, $3, $4)
            RETURNING id, email, username, password_hash
            "#,
            user.id,
            user.email,
            user.username,
            user.password_hash
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DomainError::Internal(e.to_string()))?;

        Ok(row.into())
    }
}
