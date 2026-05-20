use axum::Router;
use axum::extract::FromRef;
use axum::routing::post;

use crate::AppState;
use crate::handlers::auth_handler;

// pub fn create_auth_route() -> Router<AppState> {
//     Router::new().route("/login", post(auth_handler::login))
// }
pub fn create_auth_route<S>() -> Router<S>
where
    S: Clone + Send + Sync + 'static,
    AppState: FromRef<S>,
    // Config: FromRef<S>,
{
    Router::new().route("/login", post(auth_handler::login))
}
