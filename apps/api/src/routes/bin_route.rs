use axum::Router;
use axum::extract::FromRef;
use axum::middleware::from_fn_with_state;
use axum::routing::{delete, get, patch, post};

use crate::AppState;
use crate::handlers::bin_handler;
use crate::middlewares::authz::{
    require_bin_create, require_bin_delete, require_bin_update, require_bin_view,
};

/// Route quản lý kệ lưu trữ. Mỗi nhóm cần permission BIN_* riêng (kiểm qua route_layer).
pub fn routes<S>(state: AppState) -> Router<S>
where
    S: Clone + Send + Sync + 'static,
    AppState: FromRef<S>,
{
    let create = Router::<S>::new()
        .route("/bins", post(bin_handler::create_bin))
        .route_layer(from_fn_with_state(state.clone(), require_bin_create));

    let view = Router::<S>::new()
        .route("/bins", get(bin_handler::list_bins))
        .route("/bins/{id}", get(bin_handler::get_bin))
        .route_layer(from_fn_with_state(state.clone(), require_bin_view));

    let update = Router::<S>::new()
        .route("/bins/{id}", patch(bin_handler::update_bin))
        .route_layer(from_fn_with_state(state.clone(), require_bin_update));

    let remove = Router::<S>::new()
        .route("/bins/{id}", delete(bin_handler::delete_bin))
        .route_layer(from_fn_with_state(state, require_bin_delete));

    Router::<S>::new()
        .merge(create)
        .merge(view)
        .merge(update)
        .merge(remove)
}
