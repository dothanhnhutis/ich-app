use axum::Router;
use axum::extract::FromRef;
use axum::middleware::from_fn_with_state;
use axum::routing::{delete, get, patch, post};

use crate::AppState;
use crate::handlers::zone_handler;
use crate::middlewares::authz::{
    require_zone_create, require_zone_delete, require_zone_update, require_zone_view,
};

/// Route quản lý khu vực kho. Mỗi nhóm cần permission ZONE_* riêng (kiểm qua route_layer).
pub fn routes<S>(state: AppState) -> Router<S>
where
    S: Clone + Send + Sync + 'static,
    AppState: FromRef<S>,
{
    let create = Router::<S>::new()
        .route("/zones", post(zone_handler::create_zone))
        .route_layer(from_fn_with_state(state.clone(), require_zone_create));

    let view = Router::<S>::new()
        .route("/zones", get(zone_handler::list_zones))
        .route("/zones/{id}", get(zone_handler::get_zone))
        .route_layer(from_fn_with_state(state.clone(), require_zone_view));

    let update = Router::<S>::new()
        .route("/zones/{id}", patch(zone_handler::update_zone))
        .route_layer(from_fn_with_state(state.clone(), require_zone_update));

    let remove = Router::<S>::new()
        .route("/zones/{id}", delete(zone_handler::delete_zone))
        .route_layer(from_fn_with_state(state, require_zone_delete));

    Router::<S>::new()
        .merge(create)
        .merge(view)
        .merge(update)
        .merge(remove)
}
