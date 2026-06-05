use std::net::SocketAddr;
use std::sync::Arc;

use application::services::auth_service::AuthService;
use axum::http::header::{AUTHORIZATION, CONTENT_TYPE};
use axum::http::{HeaderValue, Method};
use axum::{Router, extract::FromRef};
use chrono::Duration;
use dotenvy::dotenv;
use infrastructure::{
    RedisSessionCache, init_db_pool, init_redis,
    repositories::{PgUserRepository, PgUserSessionRepository},
};
use shared::config::AppConfig;
use tower_http::cors::{AllowOrigin, CorsLayer};

mod errors;
mod extractor;
mod handlers;
mod middlewares;
mod routes;

/// Shared state — chứa các service đã được inject dependencies
#[derive(Clone, FromRef)]
pub struct AppState {
    pub auth_service:
        Arc<AuthService<PgUserRepository, PgUserSessionRepository, RedisSessionCache>>,
    pub cookie_secure: bool,
    pub cookie_domain: Option<String>,
}

#[tokio::main]
async fn main() {
    dotenv().ok();

    // 1. Khởi tạo Log
    tracing_subscriber::fmt::init();

    // 2. Load config từ environment
    let config = AppConfig::from_env().expect("Failed to load config");

    // 3. Kết nối Database
    let pool = init_db_pool(&config.database_url)
        .await
        .expect("Failed to connect to DB");

    // 4. Kết nối Redis (cache phiên). Bắt buộc — panic nếu không kết nối được.
    let redis_conn = init_redis(&config.redis_url)
        .await
        .expect("Failed to connect to Redis");
    let cache = RedisSessionCache::new(redis_conn);

    // 5. Dependency Injection — tạo các repository và service
    // PgPool clone rẻ (Arc bên trong) — cả hai repo dùng chung pool.
    let user_repo = PgUserRepository::new(pool.clone());
    let session_repo = PgUserSessionRepository::new(pool);
    let auth_service = Arc::new(AuthService::new(
        user_repo,
        session_repo,
        cache,
        Duration::seconds(config.session_ttl_secs),
        Duration::seconds(config.session_cache_ttl_secs),
        Duration::seconds(config.session_db_sync_secs),
    ));

    let state = AppState {
        auth_service,
        cookie_secure: config.cookie_secure,
        cookie_domain: config.cookie_domain,
    };

    // 5. CORS — phải liệt kê origin tường minh vì allow_credentials(true) không cho phép `*`.
    let origins: Vec<HeaderValue> = config
        .cors_allowed_origins
        .iter()
        .filter_map(|o| o.parse::<HeaderValue>().ok())
        .collect();
    if origins.is_empty() {
        tracing::warn!("CORS_ALLOWED_ORIGINS rỗng — không origin nào được phép gọi kèm credentials");
    }
    let cors = CorsLayer::new()
        .allow_origin(AllowOrigin::list(origins))
        .allow_credentials(true)
        .allow_methods([
            Method::GET,
            Method::POST,
            Method::PUT,
            Method::DELETE,
            Method::OPTIONS,
        ])
        .allow_headers([AUTHORIZATION, CONTENT_TYPE]);

    // 6. Build Router (CORS là layer ngoài, áp cho toàn bộ /api/v1)
    let app = Router::new()
        .nest("/api/v1", routes::create_router(state.clone()))
        .layer(cors)
        .with_state(state);

    // 7. Chạy server
    let addr = format!("{}:{}", config.server_host, config.server_port);
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    tracing::info!("Server đang chạy tại: http://{}", addr);
    // BẮT BUỘC để extractor ConnectInfo<SocketAddr> hoạt động.
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await
    .unwrap();
}
