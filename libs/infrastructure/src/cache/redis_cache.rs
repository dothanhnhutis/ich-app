use application::ports::SessionCache;
use domain::entities::CachedSession;
use application::errors::AppError;
use redis::AsyncCommands;
use redis::aio::ConnectionManager;
use uuid::Uuid;

/// Khởi tạo kết nối Redis (ConnectionManager — multiplex, tự reconnect, Clone rẻ).
pub async fn init_redis(redis_url: &str) -> anyhow::Result<ConnectionManager> {
    let client = redis::Client::open(redis_url)?;
    let conn = client.get_connection_manager().await?;
    Ok(conn)
}

#[derive(Clone)]
pub struct RedisSessionCache {
    conn: ConnectionManager,
}

impl RedisSessionCache {
    pub fn new(conn: ConnectionManager) -> Self {
        Self { conn }
    }
}

fn session_key(token_hash: &str) -> String {
    format!("session:{token_hash}")
}

fn user_key(user_id: Uuid) -> String {
    format!("user_sessions:{user_id}")
}

fn map_redis(e: redis::RedisError) -> AppError {
    AppError::Internal(e.to_string())
}

fn map_json(e: serde_json::Error) -> AppError {
    AppError::Internal(e.to_string())
}

impl SessionCache for RedisSessionCache {
    async fn get(&self, token_hash: &str) -> Result<Option<CachedSession>, AppError> {
        let mut conn = self.conn.clone();
        let raw: Option<String> = conn.get(session_key(token_hash)).await.map_err(map_redis)?;
        match raw {
            Some(s) => Ok(Some(serde_json::from_str(&s).map_err(map_json)?)),
            None => Ok(None),
        }
    }

    async fn put(
        &self,
        token_hash: &str,
        entry: &CachedSession,
        ttl_secs: i64,
    ) -> Result<(), AppError> {
        let mut conn = self.conn.clone();
        let json = serde_json::to_string(entry).map_err(map_json)?;
        let skey = session_key(token_hash);
        let ukey = user_key(entry.session.user_id);
        let ttl = ttl_secs.max(1); // SET EX 0 / EXPIRE 0 là footgun

        redis::pipe()
            .atomic()
            .set_ex(&skey, json, ttl as u64)
            .ignore()
            .sadd(&ukey, token_hash)
            .ignore()
            .expire(&ukey, ttl)
            .ignore()
            .query_async::<()>(&mut conn)
            .await
            .map_err(map_redis)?;
        Ok(())
    }

    async fn remove(&self, token_hash: &str) -> Result<(), AppError> {
        let mut conn = self.conn.clone();
        // Chỉ xóa session key; phần tử thừa trong user set tự hết hạn theo TTL.
        let _: () = conn.del(session_key(token_hash)).await.map_err(map_redis)?;
        Ok(())
    }

    async fn remove_all_for_user(&self, user_id: Uuid) -> Result<(), AppError> {
        let mut conn = self.conn.clone();
        let ukey = user_key(user_id);
        let hashes: Vec<String> = conn.smembers(&ukey).await.map_err(map_redis)?;

        let mut pipe = redis::pipe();
        for h in &hashes {
            pipe.del(session_key(h)).ignore();
        }
        pipe.del(&ukey).ignore();
        pipe.query_async::<()>(&mut conn).await.map_err(map_redis)?;
        Ok(())
    }
}
