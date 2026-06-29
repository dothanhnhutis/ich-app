use std::str::FromStr;

use domain::entities::{NewPasswordToken, PasswordToken, PasswordTokenType};
use application::errors::AppError;
use application::ports::PasswordTokenRepository;
use sqlx::PgPool;
use sqlx::types::chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(Debug, sqlx::FromRow)]
struct PasswordTokenRow {
    id: Uuid,
    user_id: Uuid,
    token_hash: String,
    #[sqlx(rename = "type")]
    token_type: String,
    expires_at: DateTime<Utc>,
    used_at: Option<DateTime<Utc>>,
    created_at: DateTime<Utc>,
}

impl TryFrom<PasswordTokenRow> for PasswordToken {
    type Error = AppError;
    fn try_from(r: PasswordTokenRow) -> Result<Self, Self::Error> {
        Ok(PasswordToken {
            id: r.id,
            user_id: r.user_id,
            token_hash: r.token_hash,
            token_type: PasswordTokenType::from_str(&r.token_type)
                .map_err(AppError::Internal)?,
            expires_at: r.expires_at,
            used_at: r.used_at,
            created_at: r.created_at,
        })
    }
}

fn map_sqlx_err(e: sqlx::Error) -> AppError {
    if let sqlx::Error::Database(db) = &e
        && db.is_unique_violation()
    {
        return AppError::Validation("Token đã tồn tại".into());
    }
    AppError::Internal(e.to_string())
}

const INSERT_TOKEN: &str = r#"
    INSERT INTO password_tokens (user_id, token_hash, type, expires_at)
    VALUES ($1, $2, $3, $4)
    RETURNING id, user_id, token_hash, type, expires_at, used_at, created_at
"#;

const FIND_ACTIVE_BY_HASH: &str = r#"
    SELECT id, user_id, token_hash, type, expires_at, used_at, created_at
    FROM password_tokens
    WHERE token_hash = $1 AND used_at IS NULL AND expires_at > NOW()
"#;

#[derive(Clone)]
pub struct PgPasswordTokenRepository {
    pool: PgPool,
}

impl PgPasswordTokenRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

impl PasswordTokenRepository for PgPasswordTokenRepository {
    async fn create(&self, t: NewPasswordToken) -> Result<PasswordToken, AppError> {
        let row: PasswordTokenRow = sqlx::query_as(INSERT_TOKEN)
            .bind(t.user_id)
            .bind(t.token_hash)
            .bind(t.token_type.as_str())
            .bind(t.expires_at)
            .fetch_one(&self.pool)
            .await
            .map_err(map_sqlx_err)?;
        PasswordToken::try_from(row)
    }

    async fn find_active_by_hash(
        &self,
        token_hash: &str,
    ) -> Result<Option<PasswordToken>, AppError> {
        let row: Option<PasswordTokenRow> = sqlx::query_as(FIND_ACTIVE_BY_HASH)
            .bind(token_hash)
            .fetch_optional(&self.pool)
            .await
            .map_err(map_sqlx_err)?;
        row.map(PasswordToken::try_from).transpose()
    }

    async fn invalidate_active(
        &self,
        user_id: Uuid,
        token_type: PasswordTokenType,
    ) -> Result<(), AppError> {
        sqlx::query(
            "UPDATE password_tokens SET used_at = NOW() WHERE user_id = $1 AND type = $2 AND used_at IS NULL",
        )
        .bind(user_id)
        .bind(token_type.as_str())
        .execute(&self.pool)
        .await
        .map_err(map_sqlx_err)?;
        Ok(())
    }
}
