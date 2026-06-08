use axum::Json;
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use serde_json::json;
use uuid::Uuid;

use application::dto::role_dto::{CreateRoleRequest, ListRolesQuery, UpdateRoleRequest};

use crate::AppState;
use crate::errors::ApiError;
use crate::extractor::ValidatedJson;

/// Tạo vai trò mới (cần ROLE_CREATE — kiểm ở middleware).
pub async fn create_role(
    State(state): State<AppState>,
    ValidatedJson(payload): ValidatedJson<CreateRoleRequest>,
) -> Result<impl IntoResponse, ApiError> {
    let res = state.role_service.create_role(payload).await?;
    Ok((StatusCode::CREATED, Json(res)))
}

/// Danh sách vai trò (lọc + phân trang) (cần ROLE_VIEW).
pub async fn list_roles(
    State(state): State<AppState>,
    Query(query): Query<ListRolesQuery>,
) -> Result<impl IntoResponse, ApiError> {
    let res = state.role_service.list_roles(query).await?;
    Ok(Json(res))
}

/// Cập nhật vai trò (cần ROLE_UPDATE).
pub async fn update_role(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    ValidatedJson(payload): ValidatedJson<UpdateRoleRequest>,
) -> Result<impl IntoResponse, ApiError> {
    let res = state.role_service.update_role(id, payload).await?;
    Ok(Json(res))
}

/// Xoá mềm vai trò (cần ROLE_DELETE).
pub async fn delete_role(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, ApiError> {
    state.role_service.delete_role(id).await?;
    Ok(Json(json!({ "message": "Đã xoá vai trò" })))
}

/// Danh sách permission của một role, nhóm theo prefix (cần ROLE_VIEW).
pub async fn get_role_permissions(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, ApiError> {
    let res = state.role_service.role_permissions_grouped(id).await?;
    Ok(Json(res))
}
