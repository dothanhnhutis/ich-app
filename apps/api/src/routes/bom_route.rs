use axum::Router;
use axum::extract::FromRef;
use axum::middleware::from_fn_with_state;
use axum::routing::{delete, get, patch, post};

use crate::AppState;
use crate::handlers::bom_handler;
use crate::middlewares::authz::{
    require_bom_create, require_bom_delete, require_bom_update, require_bom_view,
};

/// Route quản lý BOM (Hybrid). Quản lý dòng (`/boms/{id}/lines...`) = sửa BOM → BOM_UPDATE.
pub fn routes<S>(state: AppState) -> Router<S>
where
    S: Clone + Send + Sync + 'static,
    AppState: FromRef<S>,
{
    let create = Router::<S>::new()
        .route("/boms", post(bom_handler::create_bom))
        .route_layer(from_fn_with_state(state.clone(), require_bom_create));

    let view = Router::<S>::new()
        .route("/boms", get(bom_handler::list_boms))
        .route("/boms/{id}", get(bom_handler::get_bom))
        .route_layer(from_fn_with_state(state.clone(), require_bom_view));

    let update = Router::<S>::new()
        .route("/boms/{id}", patch(bom_handler::update_bom))
        .route("/boms/{id}/lines", post(bom_handler::add_line))
        .route(
            "/boms/{id}/lines/{line_id}",
            patch(bom_handler::update_line).delete(bom_handler::delete_line),
        )
        .route_layer(from_fn_with_state(state.clone(), require_bom_update));

    let remove = Router::<S>::new()
        .route("/boms/{id}", delete(bom_handler::delete_bom))
        .route_layer(from_fn_with_state(state, require_bom_delete));

    Router::<S>::new()
        .merge(create)
        .merge(view)
        .merge(update)
        .merge(remove)
}
