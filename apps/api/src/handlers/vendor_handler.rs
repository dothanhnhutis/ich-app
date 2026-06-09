use axum::Json;
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use serde_json::json;
use uuid::Uuid;

use application::dto::vendor_dto::{CreateVendorRequest, ListVendorsQuery, UpdateVendorRequest};

use crate::AppState;
use crate::errors::ApiError;
use crate::extractor::ValidatedJson;

/// Tạo nhà cung cấp mới (cần VENDOR_CREATE — kiểm ở middleware).
pub async fn create_vendor(
    State(state): State<AppState>,
    ValidatedJson(payload): ValidatedJson<CreateVendorRequest>,
) -> Result<impl IntoResponse, ApiError> {
    let res = state.vendor_service.create_vendor(payload).await?;
    Ok((StatusCode::CREATED, Json(res)))
}

/// Danh sách nhà cung cấp (lọc + phân trang + sắp xếp) (cần VENDOR_VIEW).
pub async fn list_vendors(
    State(state): State<AppState>,
    Query(query): Query<ListVendorsQuery>,
) -> Result<impl IntoResponse, ApiError> {
    let res = state.vendor_service.list_vendors(query).await?;
    Ok(Json(res))
}

/// Chi tiết một nhà cung cấp (cần VENDOR_VIEW).
pub async fn get_vendor(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, ApiError> {
    let res = state.vendor_service.get_vendor(id).await?;
    Ok(Json(res))
}

/// Cập nhật nhà cung cấp (cần VENDOR_UPDATE).
pub async fn update_vendor(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    ValidatedJson(payload): ValidatedJson<UpdateVendorRequest>,
) -> Result<impl IntoResponse, ApiError> {
    let res = state.vendor_service.update_vendor(id, payload).await?;
    Ok(Json(res))
}

/// Xoá mềm nhà cung cấp (cần VENDOR_DELETE).
pub async fn delete_vendor(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, ApiError> {
    state.vendor_service.delete_vendor(id).await?;
    Ok(Json(json!({ "message": "Đã xoá nhà cung cấp" })))
}
