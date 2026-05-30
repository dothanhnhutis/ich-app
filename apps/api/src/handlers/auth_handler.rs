use axum::Json;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;

use application::dto::auth_dto::LoginRequest;

use crate::AppState;
use crate::errors::ApiError;
use crate::extractor::ValidatedJson;

/// Thin handler — chỉ parse request, gọi service, format response
pub async fn login(
    State(state): State<AppState>,
    ValidatedJson(payload): ValidatedJson<LoginRequest>,
) -> Result<impl IntoResponse, ApiError> {
    let response = state.auth_service.login(payload).await?;

    Ok((StatusCode::OK, Json(response)))
}
