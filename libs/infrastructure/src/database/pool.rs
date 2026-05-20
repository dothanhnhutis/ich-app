use sqlx::postgres::{PgPool, PgPoolOptions};
use std::time::Duration;

pub async fn init_db_pool(database_url: &str) -> anyhow::Result<PgPool> {
    let pool = PgPoolOptions::new()
        .max_connections(20) // Phù hợp VPS 4GB
        .acquire_timeout(Duration::from_secs(3))
        .connect(database_url)
        .await?;

    Ok(pool)
}
