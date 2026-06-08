use axum::Json;
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use serde_json::json;
use uuid::Uuid;

use application::dto::zone_dto::{CreateZoneRequest, ListZonesQuery, UpdateZoneRequest};

use crate::AppState;
use crate::errors::ApiError;
use crate::extractor::ValidatedJson;

/// Tạo khu vực mới (cần ZONE_CREATE — kiểm ở middleware).
pub async fn create_zone(
    State(state): State<AppState>,
    ValidatedJson(payload): ValidatedJson<CreateZoneRequest>,
) -> Result<impl IntoResponse, ApiError> {
    let res = state.zone_service.create_zone(payload).await?;
    Ok((StatusCode::CREATED, Json(res)))
}

/// Danh sách khu vực (lọc + phân trang + sắp xếp) (cần ZONE_VIEW).
pub async fn list_zones(
    State(state): State<AppState>,
    Query(query): Query<ListZonesQuery>,
) -> Result<impl IntoResponse, ApiError> {
    let res = state.zone_service.list_zones(query).await?;
    Ok(Json(res))
}

/// Chi tiết một khu vực (cần ZONE_VIEW).
pub async fn get_zone(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, ApiError> {
    let res = state.zone_service.get_zone(id).await?;
    Ok(Json(res))
}

/// Cập nhật khu vực (cần ZONE_UPDATE).
pub async fn update_zone(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    ValidatedJson(payload): ValidatedJson<UpdateZoneRequest>,
) -> Result<impl IntoResponse, ApiError> {
    let res = state.zone_service.update_zone(id, payload).await?;
    Ok(Json(res))
}

/// Xoá mềm khu vực (cần ZONE_DELETE).
pub async fn delete_zone(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, ApiError> {
    state.zone_service.delete_zone(id).await?;
    Ok(Json(json!({ "message": "Đã xoá khu vực" })))
}
