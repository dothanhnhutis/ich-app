use axum::Json;
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use serde_json::json;
use uuid::Uuid;

use application::dto::bin_dto::{CreateBinRequest, ListBinsQuery, UpdateBinRequest};

use crate::AppState;
use crate::errors::ApiError;
use crate::extractor::ValidatedJson;

/// Tạo kệ mới (cần BIN_CREATE — kiểm ở middleware).
pub async fn create_bin(
    State(state): State<AppState>,
    ValidatedJson(payload): ValidatedJson<CreateBinRequest>,
) -> Result<impl IntoResponse, ApiError> {
    let res = state.bin_service.create_bin(payload).await?;
    Ok((StatusCode::CREATED, Json(res)))
}

/// Danh sách kệ (lọc + phân trang + sắp xếp) (cần BIN_VIEW).
pub async fn list_bins(
    State(state): State<AppState>,
    Query(query): Query<ListBinsQuery>,
) -> Result<impl IntoResponse, ApiError> {
    let res = state.bin_service.list_bins(query).await?;
    Ok(Json(res))
}

/// Chi tiết một kệ (cần BIN_VIEW).
pub async fn get_bin(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, ApiError> {
    let res = state.bin_service.get_bin(id).await?;
    Ok(Json(res))
}

/// Cập nhật kệ (cần BIN_UPDATE).
pub async fn update_bin(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    ValidatedJson(payload): ValidatedJson<UpdateBinRequest>,
) -> Result<impl IntoResponse, ApiError> {
    let res = state.bin_service.update_bin(id, payload).await?;
    Ok(Json(res))
}

/// Xoá mềm kệ (cần BIN_DELETE).
pub async fn delete_bin(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, ApiError> {
    state.bin_service.delete_bin(id).await?;
    Ok(Json(json!({ "message": "Đã xoá kệ" })))
}
