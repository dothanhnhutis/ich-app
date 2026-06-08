use axum::Router;
use axum::extract::FromRef;
use axum::middleware::from_fn_with_state;
use axum::routing::{delete, get, patch, post};

use crate::AppState;
use crate::handlers::location_handler;
use crate::middlewares::authz::{
    require_location_create, require_location_delete, require_location_update, require_location_view,
};

/// Route quản lý kho. Mỗi nhóm cần permission LOCATION_* riêng (kiểm qua route_layer).
pub fn routes<S>(state: AppState) -> Router<S>
where
    S: Clone + Send + Sync + 'static,
    AppState: FromRef<S>,
{
    let create = Router::<S>::new()
        .route("/locations", post(location_handler::create_location))
        .route_layer(from_fn_with_state(state.clone(), require_location_create));

    let view = Router::<S>::new()
        .route("/locations", get(location_handler::list_locations))
        .route("/locations/{id}", get(location_handler::get_location))
        .route_layer(from_fn_with_state(state.clone(), require_location_view));

    let update = Router::<S>::new()
        .route("/locations/{id}", patch(location_handler::update_location))
        .route_layer(from_fn_with_state(state.clone(), require_location_update));

    let remove = Router::<S>::new()
        .route("/locations/{id}", delete(location_handler::delete_location))
        .route_layer(from_fn_with_state(state, require_location_delete));

    Router::<S>::new()
        .merge(create)
        .merge(view)
        .merge(update)
        .merge(remove)
}
