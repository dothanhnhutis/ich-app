use axum::Router;
use axum::extract::FromRef;
use axum::routing::{get, post};

use crate::AppState;
use crate::handlers::item_handler;

/// Route quản lý vật tư (item master). Authz **per-type** kiểm ở tầng service —
/// KHÔNG có `route_layer` per-action; chỉ cần `require_auth` (áp ở nhóm protected)
/// để handler đọc được `AuthContext` và nạp permission codes.
pub fn routes<S>(_state: AppState) -> Router<S>
where
    S: Clone + Send + Sync + 'static,
    AppState: FromRef<S>,
{
    Router::<S>::new()
        .route(
            "/items",
            post(item_handler::create_item).get(item_handler::list_items),
        )
        .route(
            "/items/{id}",
            get(item_handler::get_item)
                .patch(item_handler::update_item)
                .delete(item_handler::delete_item),
        )
}
