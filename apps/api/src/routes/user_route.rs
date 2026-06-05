use axum::Router;
use axum::extract::FromRef;
use axum::routing::get;

use crate::AppState;
use crate::handlers::user_handler;

/// Route user cần xác thực.
pub fn routes<S>() -> Router<S>
where
    S: Clone + Send + Sync + 'static,
    AppState: FromRef<S>,
{
    Router::new().route("/users/me", get(user_handler::me))
}
