use axum::Router;
use axum::extract::FromRef;
use axum::middleware::from_fn_with_state;
use axum::routing::get;

use crate::AppState;
use crate::handlers::permission_handler;
use crate::middlewares::authz::require_role_view;

/// Route catalog permission. Cần xác thực + permission ROLE_VIEW.
pub fn routes<S>(state: AppState) -> Router<S>
where
    S: Clone + Send + Sync + 'static,
    AppState: FromRef<S>,
{
    Router::<S>::new()
        .route("/permissions", get(permission_handler::list_permissions))
        .route_layer(from_fn_with_state(state, require_role_view))
}
