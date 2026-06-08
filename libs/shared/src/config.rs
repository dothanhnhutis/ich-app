use std::env;

/// Cấu hình ứng dụng — load từ environment variables
pub struct AppConfig {
    pub database_url: String,
    pub server_host: String,
    pub server_port: u16,
    /// Thời gian sống của session (giây). Mặc định 30 ngày.
    pub session_ttl_secs: i64,
    /// Đặt cờ `Secure` cho cookie (bật ở production/HTTPS).
    pub cookie_secure: bool,
    /// Domain cho cookie (None = mặc định theo host hiện tại).
    pub cookie_domain: Option<String>,
    /// Danh sách origin được phép cho CORS (cần khi web app dùng cookie + credentials).
    pub cors_allowed_origins: Vec<String>,
    /// URL kết nối Redis (cache phiên).
    pub redis_url: String,
    /// TTL của bản cache phiên trong Redis (giây). Ngắn hơn session TTL để giới hạn staleness.
    pub session_cache_ttl_secs: i64,
    /// Khoảng tối thiểu giữa 2 lần ghi `expires_at` xuống DB (giây) — throttle.
    pub session_db_sync_secs: i64,
    /// URL kết nối RabbitMQ.
    pub rabbitmq_url: String,
    /// Tên queue chứa email job.
    pub rabbitmq_email_queue: String,
    /// Base URL của web app (dùng dựng link đặt mật khẩu).
    pub app_web_url: String,
    /// TTL của token thiết lập tài khoản INIT (giây). Mặc định 24h.
    pub password_token_ttl_secs: i64,
    /// TTL của token đặt lại mật khẩu RESET (giây). Mặc định 4h.
    pub reset_password_token_ttl_secs: i64,
}

impl AppConfig {
    pub fn from_env() -> anyhow::Result<Self> {
        Ok(Self {
            database_url: env::var("DATABASE_URL")
                .map_err(|_| anyhow::anyhow!("DATABASE_URL must be set"))?,
            server_host: env::var("SERVER_HOST").unwrap_or_else(|_| "0.0.0.0".into()),
            server_port: env::var("SERVER_PORT")
                .unwrap_or_else(|_| "8080".into())
                .parse()
                .map_err(|_| anyhow::anyhow!("SERVER_PORT must be a valid port number"))?,
            session_ttl_secs: env::var("SESSION_TTL_SECS")
                .unwrap_or_else(|_| "2592000".into()) // 30 ngày
                .parse()
                .map_err(|_| anyhow::anyhow!("SESSION_TTL_SECS must be an integer (seconds)"))?,
            cookie_secure: env::var("COOKIE_SECURE")
                .map(|v| v == "true" || v == "1")
                .unwrap_or(false),
            cookie_domain: env::var("COOKIE_DOMAIN").ok(),
            cors_allowed_origins: env::var("CORS_ALLOWED_ORIGINS")
                .unwrap_or_default()
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect(),
            redis_url: env::var("REDIS_URL").unwrap_or_else(|_| "redis://127.0.0.1:6379".into()),
            session_cache_ttl_secs: env::var("SESSION_CACHE_TTL_SECS")
                .unwrap_or_else(|_| "3600".into())
                .parse()
                .map_err(|_| anyhow::anyhow!("SESSION_CACHE_TTL_SECS must be an integer (seconds)"))?,
            session_db_sync_secs: env::var("SESSION_DB_SYNC_SECS")
                .unwrap_or_else(|_| "60".into())
                .parse()
                .map_err(|_| anyhow::anyhow!("SESSION_DB_SYNC_SECS must be an integer (seconds)"))?,
            rabbitmq_url: env::var("RABBITMQ_URL")
                .unwrap_or_else(|_| "amqp://guest:guest@127.0.0.1:5672/%2f".into()),
            rabbitmq_email_queue: env::var("RABBITMQ_EMAIL_QUEUE")
                .unwrap_or_else(|_| "email_jobs".into()),
            app_web_url: env::var("APP_WEB_URL").unwrap_or_else(|_| "http://localhost:5173".into()),
            password_token_ttl_secs: env::var("PASSWORD_TOKEN_TTL_SECS")
                .unwrap_or_else(|_| "86400".into())
                .parse()
                .map_err(|_| anyhow::anyhow!("PASSWORD_TOKEN_TTL_SECS must be an integer (seconds)"))?,
            reset_password_token_ttl_secs: env::var("RESET_PASSWORD_TOKEN_TTL_SECS")
                .unwrap_or_else(|_| "14400".into()) // 4h
                .parse()
                .map_err(|_| {
                    anyhow::anyhow!("RESET_PASSWORD_TOKEN_TTL_SECS must be an integer (seconds)")
                })?,
        })
    }
}
