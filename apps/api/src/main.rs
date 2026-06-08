use std::net::SocketAddr;
use std::sync::Arc;

use application::services::account_service::AccountService;
use application::services::auth_service::AuthService;
use application::services::bin_service::BinService;
use application::services::location_service::LocationService;
use application::services::role_service::RoleService;
use application::services::user_service::UserService;
use application::services::zone_service::ZoneService;
use axum::http::header::{AUTHORIZATION, CONTENT_TYPE};
use axum::http::{HeaderValue, Method};
use axum::{Router, extract::FromRef};
use chrono::Duration;
use dotenvy::dotenv;
use infrastructure::{
    RabbitEmailPublisher, RedisSessionCache, init_db_pool, init_redis,
    repositories::{
        PgBinRepository, PgLocationRepository, PgPasswordTokenRepository, PgRoleRepository,
        PgUserRepository, PgUserSessionRepository, PgZoneRepository,
    },
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
    pub user_service: Arc<
        UserService<
            PgUserRepository,
            PgRoleRepository,
            PgPasswordTokenRepository,
            RabbitEmailPublisher,
        >,
    >,
    pub account_service: Arc<
        AccountService<PgUserRepository, PgPasswordTokenRepository, RabbitEmailPublisher>,
    >,
    pub role_service: Arc<RoleService<PgRoleRepository>>,
    pub location_service: Arc<LocationService<PgLocationRepository>>,
    pub zone_service: Arc<ZoneService<PgZoneRepository, PgLocationRepository>>,
    pub bin_service: Arc<BinService<PgBinRepository, PgZoneRepository>>,
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

    // 5. Kết nối RabbitMQ (publisher email job). Bắt buộc.
    let email_publisher =
        RabbitEmailPublisher::connect(&config.rabbitmq_url, &config.rabbitmq_email_queue)
            .await
            .expect("Failed to connect to RabbitMQ");

    // 6. Dependency Injection — PgPool/repo Clone rẻ (Arc bên trong), dùng chung pool.
    let user_repo = PgUserRepository::new(pool.clone());
    let session_repo = PgUserSessionRepository::new(pool.clone());
    let role_repo = PgRoleRepository::new(pool.clone());
    let token_repo = PgPasswordTokenRepository::new(pool.clone());
    let location_repo = PgLocationRepository::new(pool.clone());
    let zone_repo = PgZoneRepository::new(pool.clone());
    let bin_repo = PgBinRepository::new(pool.clone());

    let auth_service = Arc::new(AuthService::new(
        user_repo.clone(),
        session_repo,
        cache,
        Duration::seconds(config.session_ttl_secs),
        Duration::seconds(config.session_cache_ttl_secs),
        Duration::seconds(config.session_db_sync_secs),
    ));
    let user_service = Arc::new(UserService::new(
        user_repo.clone(),
        role_repo.clone(),
        token_repo.clone(),
        email_publisher.clone(),
        config.app_web_url.clone(),
        config.password_token_ttl_secs,
    ));
    let account_service = Arc::new(AccountService::new(
        user_repo,
        token_repo,
        email_publisher,
        config.app_web_url.clone(),
        config.reset_password_token_ttl_secs,
    ));
    let role_service = Arc::new(RoleService::new(role_repo));
    let location_service = Arc::new(LocationService::new(location_repo.clone()));
    let zone_service = Arc::new(ZoneService::new(zone_repo.clone(), location_repo));
    let bin_service = Arc::new(BinService::new(bin_repo, zone_repo));

    let state = AppState {
        auth_service,
        user_service,
        account_service,
        role_service,
        location_service,
        zone_service,
        bin_service,
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
            Method::PATCH,
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
