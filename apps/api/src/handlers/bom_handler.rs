use axum::Json;
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use serde_json::json;
use uuid::Uuid;

use application::dto::bom_dto::{
    AddBomLineRequest, CreateBomRequest, ListBomsQuery, UpdateBomLineRequest, UpdateBomRequest,
};

use crate::AppState;
use crate::errors::ApiError;
use crate::extractor::ValidatedJson;

/// Tạo BOM kèm dòng (transaction) (cần BOM_CREATE — kiểm ở middleware).
pub async fn create_bom(
    State(state): State<AppState>,
    ValidatedJson(payload): ValidatedJson<CreateBomRequest>,
) -> Result<impl IntoResponse, ApiError> {
    let res = state.bom_service.create_bom(payload).await?;
    Ok((StatusCode::CREATED, Json(res)))
}

/// Danh sách BOM (lọc + phân trang + sắp xếp) (cần BOM_VIEW).
pub async fn list_boms(
    State(state): State<AppState>,
    Query(query): Query<ListBomsQuery>,
) -> Result<impl IntoResponse, ApiError> {
    let res = state.bom_service.list_boms(query).await?;
    Ok(Json(res))
}

/// Chi tiết BOM kèm dòng (cần BOM_VIEW).
pub async fn get_bom(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, ApiError> {
    let res = state.bom_service.get_bom(id).await?;
    Ok(Json(res))
}

/// Cập nhật BOM header (cần BOM_UPDATE).
pub async fn update_bom(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    ValidatedJson(payload): ValidatedJson<UpdateBomRequest>,
) -> Result<impl IntoResponse, ApiError> {
    let res = state.bom_service.update_bom(id, payload).await?;
    Ok(Json(res))
}

/// Xoá mềm BOM + cascade dòng (cần BOM_DELETE).
pub async fn delete_bom(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, ApiError> {
    state.bom_service.delete_bom(id).await?;
    Ok(Json(json!({ "message": "Đã xoá BOM" })))
}

/// Thêm một dòng vào BOM (cần BOM_UPDATE).
pub async fn add_line(
    State(state): State<AppState>,
    Path(bom_id): Path<Uuid>,
    ValidatedJson(payload): ValidatedJson<AddBomLineRequest>,
) -> Result<impl IntoResponse, ApiError> {
    let res = state.bom_service.add_line(bom_id, payload).await?;
    Ok((StatusCode::CREATED, Json(res)))
}

/// Cập nhật một dòng BOM (cần BOM_UPDATE).
pub async fn update_line(
    State(state): State<AppState>,
    Path((bom_id, line_id)): Path<(Uuid, Uuid)>,
    ValidatedJson(payload): ValidatedJson<UpdateBomLineRequest>,
) -> Result<impl IntoResponse, ApiError> {
    let res = state.bom_service.update_line(bom_id, line_id, payload).await?;
    Ok(Json(res))
}

/// Xoá mềm một dòng BOM (cần BOM_UPDATE).
pub async fn delete_line(
    State(state): State<AppState>,
    Path((bom_id, line_id)): Path<(Uuid, Uuid)>,
) -> Result<impl IntoResponse, ApiError> {
    state.bom_service.delete_line(bom_id, line_id).await?;
    Ok(Json(json!({ "message": "Đã xoá dòng BOM" })))
}
