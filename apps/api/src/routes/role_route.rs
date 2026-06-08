use axum::Router;
use axum::extract::FromRef;
use axum::middleware::from_fn_with_state;
use axum::routing::{delete, get, patch, post};

use crate::AppState;
use crate::handlers::role_handler;
use crate::middlewares::authz::{
    require_role_create, require_role_delete, require_role_update, require_role_view,
};

/// Route quản lý vai trò. Mỗi method cần permission ROLE_* riêng (kiểm qua route_layer).
pub fn routes<S>(state: AppState) -> Router<S>
where
    S: Clone + Send + Sync + 'static,
    AppState: FromRef<S>,
{
    let create = Router::<S>::new()
        .route("/roles", post(role_handler::create_role))
        .route_layer(from_fn_with_state(state.clone(), require_role_create));

    let view = Router::<S>::new()
        .route("/roles", get(role_handler::list_roles))
        .route(
            "/roles/{id}/permissions",
            get(role_handler::get_role_permissions),
        )
        .route_layer(from_fn_with_state(state.clone(), require_role_view));

    let update = Router::<S>::new()
        .route("/roles/{id}", patch(role_handler::update_role))
        .route_layer(from_fn_with_state(state.clone(), require_role_update));

    let remove = Router::<S>::new()
        .route("/roles/{id}", delete(role_handler::delete_role))
        .route_layer(from_fn_with_state(state, require_role_delete));

    Router::<S>::new()
        .merge(create)
        .merge(view)
        .merge(update)
        .merge(remove)
}
