use axum::Router;
use axum::extract::FromRef;
use axum::routing::post;

use crate::AppState;
use crate::handlers::auth_handler;

/// Route công khai (không cần xác thực).
pub fn public_routes<S>() -> Router<S>
where
    S: Clone + Send + Sync + 'static,
    AppState: FromRef<S>,
{
    Router::new().route("/auth/login", post(auth_handler::login))
}

/// Route cần xác thực (middleware require_auth được áp ở routes::create_router).
pub fn protected_routes<S>() -> Router<S>
where
    S: Clone + Send + Sync + 'static,
    AppState: FromRef<S>,
{
    Router::new()
        .route("/auth/logout", post(auth_handler::logout))
        .route("/auth/logout-all", post(auth_handler::logout_all))
}
