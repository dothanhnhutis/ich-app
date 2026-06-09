use axum::Extension;
use axum::extract::{Request, State};
use axum::middleware::Next;
use axum::response::Response;
use uuid::Uuid;

use application::errors::AppError;

use crate::AppState;
use crate::errors::ApiError;
use crate::middlewares::auth::AuthContext;

/// Kiểm tra user có `code` trong tập permission không — thiếu thì 403.
/// Permission được nạp mỗi request (không cache) để authz luôn cập nhật.
async fn ensure(state: &AppState, user_id: Uuid, code: &str) -> Result<(), ApiError> {
    let codes = state.user_service.permission_codes(user_id).await?;
    if !codes.iter().any(|c| c == code) {
        return Err(AppError::Forbidden("Bạn không có quyền thực hiện thao tác này".into()).into());
    }
    Ok(())
}

/// `AuthContext` do `require_auth` (layer ngoài) gắn trước đó.
pub async fn require_user_create(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthContext>,
    req: Request,
    next: Next,
) -> Result<Response, ApiError> {
    ensure(&state, auth.user.id, "USER_CREATE").await?;
    Ok(next.run(req).await)
}

pub async fn require_user_view(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthContext>,
    req: Request,
    next: Next,
) -> Result<Response, ApiError> {
    ensure(&state, auth.user.id, "USER_VIEW").await?;
    Ok(next.run(req).await)
}

pub async fn require_user_update(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthContext>,
    req: Request,
    next: Next,
) -> Result<Response, ApiError> {
    ensure(&state, auth.user.id, "USER_UPDATE").await?;
    Ok(next.run(req).await)
}

pub async fn require_user_delete(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthContext>,
    req: Request,
    next: Next,
) -> Result<Response, ApiError> {
    ensure(&state, auth.user.id, "USER_DELETE").await?;
    Ok(next.run(req).await)
}

pub async fn require_role_view(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthContext>,
    req: Request,
    next: Next,
) -> Result<Response, ApiError> {
    ensure(&state, auth.user.id, "ROLE_VIEW").await?;
    Ok(next.run(req).await)
}

pub async fn require_role_create(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthContext>,
    req: Request,
    next: Next,
) -> Result<Response, ApiError> {
    ensure(&state, auth.user.id, "ROLE_CREATE").await?;
    Ok(next.run(req).await)
}

pub async fn require_role_update(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthContext>,
    req: Request,
    next: Next,
) -> Result<Response, ApiError> {
    ensure(&state, auth.user.id, "ROLE_UPDATE").await?;
    Ok(next.run(req).await)
}

pub async fn require_role_delete(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthContext>,
    req: Request,
    next: Next,
) -> Result<Response, ApiError> {
    ensure(&state, auth.user.id, "ROLE_DELETE").await?;
    Ok(next.run(req).await)
}

pub async fn require_location_view(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthContext>,
    req: Request,
    next: Next,
) -> Result<Response, ApiError> {
    ensure(&state, auth.user.id, "LOCATION_VIEW").await?;
    Ok(next.run(req).await)
}

pub async fn require_location_create(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthContext>,
    req: Request,
    next: Next,
) -> Result<Response, ApiError> {
    ensure(&state, auth.user.id, "LOCATION_CREATE").await?;
    Ok(next.run(req).await)
}

pub async fn require_location_update(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthContext>,
    req: Request,
    next: Next,
) -> Result<Response, ApiError> {
    ensure(&state, auth.user.id, "LOCATION_UPDATE").await?;
    Ok(next.run(req).await)
}

pub async fn require_location_delete(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthContext>,
    req: Request,
    next: Next,
) -> Result<Response, ApiError> {
    ensure(&state, auth.user.id, "LOCATION_DELETE").await?;
    Ok(next.run(req).await)
}

pub async fn require_zone_view(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthContext>,
    req: Request,
    next: Next,
) -> Result<Response, ApiError> {
    ensure(&state, auth.user.id, "ZONE_VIEW").await?;
    Ok(next.run(req).await)
}

pub async fn require_zone_create(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthContext>,
    req: Request,
    next: Next,
) -> Result<Response, ApiError> {
    ensure(&state, auth.user.id, "ZONE_CREATE").await?;
    Ok(next.run(req).await)
}

pub async fn require_zone_update(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthContext>,
    req: Request,
    next: Next,
) -> Result<Response, ApiError> {
    ensure(&state, auth.user.id, "ZONE_UPDATE").await?;
    Ok(next.run(req).await)
}

pub async fn require_zone_delete(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthContext>,
    req: Request,
    next: Next,
) -> Result<Response, ApiError> {
    ensure(&state, auth.user.id, "ZONE_DELETE").await?;
    Ok(next.run(req).await)
}

pub async fn require_bin_view(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthContext>,
    req: Request,
    next: Next,
) -> Result<Response, ApiError> {
    ensure(&state, auth.user.id, "BIN_VIEW").await?;
    Ok(next.run(req).await)
}

pub async fn require_bin_create(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthContext>,
    req: Request,
    next: Next,
) -> Result<Response, ApiError> {
    ensure(&state, auth.user.id, "BIN_CREATE").await?;
    Ok(next.run(req).await)
}

pub async fn require_bin_update(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthContext>,
    req: Request,
    next: Next,
) -> Result<Response, ApiError> {
    ensure(&state, auth.user.id, "BIN_UPDATE").await?;
    Ok(next.run(req).await)
}

pub async fn require_bin_delete(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthContext>,
    req: Request,
    next: Next,
) -> Result<Response, ApiError> {
    ensure(&state, auth.user.id, "BIN_DELETE").await?;
    Ok(next.run(req).await)
}

pub async fn require_vendor_view(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthContext>,
    req: Request,
    next: Next,
) -> Result<Response, ApiError> {
    ensure(&state, auth.user.id, "VENDOR_VIEW").await?;
    Ok(next.run(req).await)
}

pub async fn require_vendor_create(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthContext>,
    req: Request,
    next: Next,
) -> Result<Response, ApiError> {
    ensure(&state, auth.user.id, "VENDOR_CREATE").await?;
    Ok(next.run(req).await)
}

pub async fn require_vendor_update(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthContext>,
    req: Request,
    next: Next,
) -> Result<Response, ApiError> {
    ensure(&state, auth.user.id, "VENDOR_UPDATE").await?;
    Ok(next.run(req).await)
}

pub async fn require_vendor_delete(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthContext>,
    req: Request,
    next: Next,
) -> Result<Response, ApiError> {
    ensure(&state, auth.user.id, "VENDOR_DELETE").await?;
    Ok(next.run(req).await)
}
