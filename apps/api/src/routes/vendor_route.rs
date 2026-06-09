use axum::Router;
use axum::extract::FromRef;
use axum::middleware::from_fn_with_state;
use axum::routing::{delete, get, patch, post};

use crate::AppState;
use crate::handlers::vendor_handler;
use crate::middlewares::authz::{
    require_vendor_create, require_vendor_delete, require_vendor_update, require_vendor_view,
};

/// Route quản lý nhà cung cấp. Mỗi nhóm cần permission VENDOR_* riêng (kiểm qua route_layer).
pub fn routes<S>(state: AppState) -> Router<S>
where
    S: Clone + Send + Sync + 'static,
    AppState: FromRef<S>,
{
    let create = Router::<S>::new()
        .route("/vendors", post(vendor_handler::create_vendor))
        .route_layer(from_fn_with_state(state.clone(), require_vendor_create));

    let view = Router::<S>::new()
        .route("/vendors", get(vendor_handler::list_vendors))
        .route("/vendors/{id}", get(vendor_handler::get_vendor))
        .route_layer(from_fn_with_state(state.clone(), require_vendor_view));

    let update = Router::<S>::new()
        .route("/vendors/{id}", patch(vendor_handler::update_vendor))
        .route_layer(from_fn_with_state(state.clone(), require_vendor_update));

    let remove = Router::<S>::new()
        .route("/vendors/{id}", delete(vendor_handler::delete_vendor))
        .route_layer(from_fn_with_state(state, require_vendor_delete));

    Router::<S>::new()
        .merge(create)
        .merge(view)
        .merge(update)
        .merge(remove)
}
