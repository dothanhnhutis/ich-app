mod auth_route;
mod bin_route;
mod bom_route;
mod item_route;
mod location_route;
mod permission_route;
mod role_route;
mod user_route;
mod vendor_route;
mod zone_route;

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
        .merge(user_route::routes::<S>(state.clone()))
        .merge(permission_route::routes::<S>(state.clone()))
        .merge(role_route::routes::<S>(state.clone()))
        .merge(location_route::routes::<S>(state.clone()))
        .merge(zone_route::routes::<S>(state.clone()))
        .merge(bin_route::routes::<S>(state.clone()))
        .merge(vendor_route::routes::<S>(state.clone()))
        .merge(item_route::routes::<S>(state.clone()))
        .merge(bom_route::routes::<S>(state.clone()))
        // route_layer: middleware chỉ chạy trên các route protected đã khai báo ở trên.
        .route_layer(from_fn_with_state(state, require_auth));

    public.merge(protected)
}
