use domain::entities::User;
use domain::errors::DomainError;
use domain::repositories::UserRepository;
use sqlx::PgPool;
use uuid::Uuid;

/// Struct riêng cho DB layer — tách biệt khỏi domain entity
#[derive(Debug, sqlx::FromRow)]
struct UserRow {
    pub id: Uuid,
    pub email: String,
    pub username: String,
    pub password_hash: String,
}

/// Mapping từ DB row → Domain entity
impl From<UserRow> for User {
    fn from(row: UserRow) -> Self {
        Self {
            id: row.id,
            email: row.email,
            username: row.username,
            password_hash: row.password_hash,
        }
    }
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
        let row = sqlx::query_as!(
            UserRow,
            r#"
            SELECT id, email, username, password_hash
            FROM users
            WHERE email = $1
            "#,
            email
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DomainError::Internal(e.to_string()))?;

        Ok(row.map(|r| r.into()))
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
