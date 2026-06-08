use axum::Json;
use axum::extract::State;
use axum::response::IntoResponse;

use crate::AppState;
use crate::errors::ApiError;

/// Danh mục toàn bộ permission, nhóm theo prefix của code (cần permission ROLE_VIEW — kiểm ở middleware).
pub async fn list_permissions(
    State(state): State<AppState>,
) -> Result<impl IntoResponse, ApiError> {
    let res = state.user_service.list_permissions_grouped().await?;
    Ok(Json(res))
}
