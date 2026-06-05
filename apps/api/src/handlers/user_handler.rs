use axum::Json;
use axum::response::IntoResponse;
use axum::Extension;

use application::dto::user_dto::UserResponse;

use crate::middlewares::auth::AuthContext;

/// Trả về thông tin user đang đăng nhập (AuthContext do middleware require_auth gắn).
pub async fn me(Extension(auth): Extension<AuthContext>) -> impl IntoResponse {
    Json(UserResponse::from(auth.user))
}
