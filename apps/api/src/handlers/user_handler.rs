use axum::Json;
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Extension;
use serde_json::json;
use uuid::Uuid;

use application::dto::user_dto::{
    CreateUserRequest, ListUsersQuery, UpdateUserRequest, UserResponse,
};

use crate::AppState;
use crate::errors::ApiError;
use crate::extractor::ValidatedJson;
use crate::middlewares::auth::AuthContext;

/// Trả về thông tin user đang đăng nhập (AuthContext do middleware require_auth gắn).
pub async fn me(Extension(auth): Extension<AuthContext>) -> impl IntoResponse {
    Json(UserResponse::from(auth.user))
}

/// Admin tạo user mới (cần permission USER_CREATE — đã kiểm ở middleware).
pub async fn create_user(
    State(state): State<AppState>,
    ValidatedJson(payload): ValidatedJson<CreateUserRequest>,
) -> Result<impl IntoResponse, ApiError> {
    let res = state.user_service.create_user(payload).await?;
    Ok((StatusCode::CREATED, Json(res)))
}

/// Danh sách user (lọc + phân trang + sắp xếp) (cần USER_VIEW).
pub async fn list_users(
    State(state): State<AppState>,
    Query(query): Query<ListUsersQuery>,
) -> Result<impl IntoResponse, ApiError> {
    let res = state.user_service.list_users(query).await?;
    Ok(Json(res))
}

/// Danh sách role được gán cho một user (cần USER_VIEW).
pub async fn get_user_roles(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, ApiError> {
    let res = state.user_service.user_roles(id).await?;
    Ok(Json(res))
}

/// Cập nhật username/status của user (cần USER_UPDATE). Vô hiệu hoá → thu hồi phiên ngay.
pub async fn update_user(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    ValidatedJson(payload): ValidatedJson<UpdateUserRequest>,
) -> Result<impl IntoResponse, ApiError> {
    let res = state.user_service.update_user(id, payload).await?;
    if res.status == "DEACTIVATED" {
        state.auth_service.logout_all(id).await?;
    }
    Ok(Json(res))
}

/// Xoá mềm user (cần USER_DELETE) + thu hồi mọi phiên đăng nhập của user đó.
pub async fn delete_user(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, ApiError> {
    state.user_service.delete_user(id).await?;
    state.auth_service.logout_all(id).await?;
    Ok(Json(json!({ "message": "Đã xoá người dùng" })))
}

/// Admin gửi lại email thiết lập tài khoản cho user chưa kích hoạt (cần USER_CREATE).
pub async fn resend_setup(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, ApiError> {
    state.user_service.resend_setup(id).await?;
    Ok(Json(json!({ "message": "Đã gửi lại email thiết lập tài khoản" })))
}
