use std::sync::Arc;

use application::services::auth_service::AuthService;
use axum::{Router, extract::FromRef};
use dotenvy::dotenv;
use infrastructure::{init_db_pool, repositories::PgUserRepository};
use shared::config::AppConfig;

mod errors;
mod extractor;
mod handlers;
mod routes;

/// Shared state — chứa các service đã được inject dependencies
#[derive(Clone, FromRef)]
pub struct AppState {
    pub auth_service: Arc<AuthService<PgUserRepository>>,
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

    // 4. Dependency Injection — tạo các repository và service
    let user_repo = PgUserRepository::new(pool);
    let auth_service = Arc::new(AuthService::new(user_repo));

    let state = AppState { auth_service };

    // 5. Build Router
    let app = Router::new()
        .nest("/api/v1", routes::create_router())
        .with_state(state);

    // 6. Chạy server
    let addr = format!("{}:{}", config.server_host, config.server_port);
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    tracing::info!("Server đang chạy tại: http://{}", addr);
    axum::serve(listener, app).await.unwrap();
}
