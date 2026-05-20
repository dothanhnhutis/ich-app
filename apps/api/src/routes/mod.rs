mod auth_route;

use axum::{Router, extract::FromRef};

use crate::AppState;

// pub fn create_router() -> Router<AppState> {
//     Router::new().nest("/auth", auth_route::create_auth_route())
// }

pub fn create_router<S>() -> Router<S>
where
    S: Clone + Send + Sync + 'static,
    AppState: FromRef<S>,
{
    Router::new().nest("/auth", auth_route::create_auth_route())
}
