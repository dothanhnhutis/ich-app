use axum::Router;
use axum::extract::FromRef;
use axum::middleware::from_fn_with_state;
use axum::routing::{delete, get, patch, post};

use crate::AppState;
use crate::handlers::user_handler;
use crate::middlewares::authz::{
    require_user_create, require_user_delete, require_user_update, require_user_view,
};

/// Route user cần xác thực. Mỗi nhóm cần permission USER_* riêng (kiểm qua route_layer).
pub fn routes<S>(state: AppState) -> Router<S>
where
    S: Clone + Send + Sync + 'static,
    AppState: FromRef<S>,
{
    let manage = Router::<S>::new()
        .route("/users", post(user_handler::create_user))
        .route(
            "/users/{id}/resend-setup",
            post(user_handler::resend_setup),
        )
        .route_layer(from_fn_with_state(state.clone(), require_user_create));

    let view = Router::<S>::new()
        .route("/users", get(user_handler::list_users))
        .route("/users/{id}/roles", get(user_handler::get_user_roles))
        .route_layer(from_fn_with_state(state.clone(), require_user_view));

    let update = Router::<S>::new()
        .route("/users/{id}", patch(user_handler::update_user))
        .route_layer(from_fn_with_state(state.clone(), require_user_update));

    let remove = Router::<S>::new()
        .route("/users/{id}", delete(user_handler::delete_user))
        .route_layer(from_fn_with_state(state, require_user_delete));

    Router::<S>::new()
        .route("/users/me", get(user_handler::me))
        .merge(manage)
        .merge(view)
        .merge(update)
        .merge(remove)
}
