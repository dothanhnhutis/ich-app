use axum::Json;
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Extension;
use serde_json::json;
use uuid::Uuid;

use application::dto::item_dto::{CreateItemRequest, ListItemsQuery, UpdateItemRequest};

use crate::AppState;
use crate::errors::ApiError;
use crate::extractor::ValidatedJson;
use crate::middlewares::auth::AuthContext;

/// Authz per-type ở tầng service → handler nạp permission codes của user rồi truyền vào.
/// (Không có route_layer per-action như các resource khác.)
async fn user_codes(state: &AppState, auth: &AuthContext) -> Result<Vec<String>, ApiError> {
    Ok(state.user_service.permission_codes(auth.user.id).await?)
}

/// Tạo vật tư (cần `{TYPE}_CREATE`).
pub async fn create_item(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthContext>,
    ValidatedJson(payload): ValidatedJson<CreateItemRequest>,
) -> Result<impl IntoResponse, ApiError> {
    let codes = user_codes(&state, &auth).await?;
    let res = state.item_service.create_item(&codes, payload).await?;
    Ok((StatusCode::CREATED, Json(res)))
}

/// Danh sách vật tư (chỉ trả loại user có `{TYPE}_VIEW`).
pub async fn list_items(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthContext>,
    Query(query): Query<ListItemsQuery>,
) -> Result<impl IntoResponse, ApiError> {
    let codes = user_codes(&state, &auth).await?;
    let res = state.item_service.list_items(&codes, query).await?;
    Ok(Json(res))
}

/// Chi tiết vật tư (cần `{TYPE}_VIEW`).
pub async fn get_item(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthContext>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, ApiError> {
    let codes = user_codes(&state, &auth).await?;
    let res = state.item_service.get_item(&codes, id).await?;
    Ok(Json(res))
}

/// Cập nhật vật tư (cần `{TYPE}_UPDATE`).
pub async fn update_item(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthContext>,
    Path(id): Path<Uuid>,
    ValidatedJson(payload): ValidatedJson<UpdateItemRequest>,
) -> Result<impl IntoResponse, ApiError> {
    let codes = user_codes(&state, &auth).await?;
    let res = state.item_service.update_item(&codes, id, payload).await?;
    Ok(Json(res))
}

/// Xoá mềm vật tư (cần `{TYPE}_DELETE`).
pub async fn delete_item(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthContext>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, ApiError> {
    let codes = user_codes(&state, &auth).await?;
    state.item_service.delete_item(&codes, id).await?;
    Ok(Json(json!({ "message": "Đã xoá vật tư" })))
}
