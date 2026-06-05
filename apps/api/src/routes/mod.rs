mod auth_route;
mod user_route;

use axum::{Router, extract::FromRef, middleware::from_fn_with_state};

use crate::AppState;
use crate::middlewares::auth::require_auth;

pub fn create_router<S>(state: AppState) -> Router<S>
where
    S: Clone + Send + Sync + 'static,
    AppState: FromRef<S>,
{
    let public = auth_route::public_routes::<S>();

    let protected = auth_route::protected_routes::<S>()
        .merge(user_route::routes::<S>())
        // route_layer: middleware chỉ chạy trên các route protected đã khai báo ở trên.
        .route_layer(from_fn_with_state(state, require_auth));

    public.merge(protected)
}
