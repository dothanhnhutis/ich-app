use axum::Json;
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use serde_json::json;
use uuid::Uuid;

use application::dto::location_dto::{
    CreateLocationRequest, ListLocationsQuery, UpdateLocationRequest,
};

use crate::AppState;
use crate::errors::ApiError;
use crate::extractor::ValidatedJson;

/// Tạo kho mới (cần LOCATION_CREATE — kiểm ở middleware).
pub async fn create_location(
    State(state): State<AppState>,
    ValidatedJson(payload): ValidatedJson<CreateLocationRequest>,
) -> Result<impl IntoResponse, ApiError> {
    let res = state.location_service.create_location(payload).await?;
    Ok((StatusCode::CREATED, Json(res)))
}

/// Danh sách kho (lọc + phân trang + sắp xếp) (cần LOCATION_VIEW).
pub async fn list_locations(
    State(state): State<AppState>,
    Query(query): Query<ListLocationsQuery>,
) -> Result<impl IntoResponse, ApiError> {
    let res = state.location_service.list_locations(query).await?;
    Ok(Json(res))
}

/// Chi tiết một kho (cần LOCATION_VIEW).
pub async fn get_location(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, ApiError> {
    let res = state.location_service.get_location(id).await?;
    Ok(Json(res))
}

/// Cập nhật kho (cần LOCATION_UPDATE).
pub async fn update_location(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    ValidatedJson(payload): ValidatedJson<UpdateLocationRequest>,
) -> Result<impl IntoResponse, ApiError> {
    let res = state.location_service.update_location(id, payload).await?;
    Ok(Json(res))
}

/// Xoá mềm kho (cần LOCATION_DELETE).
pub async fn delete_location(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, ApiError> {
    state.location_service.delete_location(id).await?;
    Ok(Json(json!({ "message": "Đã xoá kho" })))
}
