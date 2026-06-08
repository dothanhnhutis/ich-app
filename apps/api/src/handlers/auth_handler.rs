use std::net::{IpAddr, SocketAddr};

use axum::extract::{ConnectInfo, State};
use axum::http::HeaderMap;
use axum::response::IntoResponse;
use axum::{Extension, Json};
use axum_extra::extract::cookie::{Cookie, CookieJar, SameSite};
use serde_json::json;

use application::dto::auth_dto::{
    ClientContext, ForgotPasswordRequest, LoginRequest, SetPasswordRequest, SetupAccountRequest,
};

use crate::AppState;
use crate::errors::ApiError;
use crate::extractor::ValidatedJson;
use crate::middlewares::auth::AuthContext;

/// Thin handler — gom metadata HTTP, gọi service, set cookie + trả JSON.
pub async fn login(
    ConnectInfo(peer): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    State(state): State<AppState>,
    jar: CookieJar,
    ValidatedJson(payload): ValidatedJson<LoginRequest>, // body extractor PHẢI ở cuối
) -> Result<impl IntoResponse, ApiError> {
    let ctx = ClientContext {
        user_agent: user_agent(&headers),
        ip_address: Some(client_ip(&headers, peer)),
    };

    let response = state.auth_service.login(payload, ctx).await?;

    let cookie = session_cookie(
        response.session.clone(),
        state.cookie_secure,
        state.cookie_domain.as_deref(),
        Some(time::Duration::seconds(response.expires_in)),
    );

    // (CookieJar, Json): jar là IntoResponseParts nên đứng trước body.
    Ok((jar.add(cookie), Json(response)))
}

/// Đăng xuất phiên hiện tại + xóa cookie.
pub async fn logout(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthContext>,
    jar: CookieJar,
) -> Result<impl IntoResponse, ApiError> {
    state
        .auth_service
        .logout(auth.session.id, &auth.session.token_hash)
        .await?;

    let removal = session_cookie(
        String::new(),
        state.cookie_secure,
        state.cookie_domain.as_deref(),
        None,
    );
    Ok((
        jar.remove(removal),
        Json(json!({ "message": "Đã đăng xuất" })),
    ))
}

/// Đăng xuất tất cả thiết bị của user + xóa cookie hiện tại.
pub async fn logout_all(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthContext>,
    jar: CookieJar,
) -> Result<impl IntoResponse, ApiError> {
    state.auth_service.logout_all(auth.user.id).await?;

    let removal = session_cookie(
        String::new(),
        state.cookie_secure,
        state.cookie_domain.as_deref(),
        None,
    );
    Ok((
        jar.remove(removal),
        Json(json!({ "message": "Đã đăng xuất khỏi tất cả thiết bị" })),
    ))
}

/// Đặt mật khẩu từ token INIT trong email (public — token tự xác thực).
/// Thiết lập tài khoản (token INIT): nhập username + mật khẩu khi admin tạo tài khoản.
pub async fn setup_account(
    State(state): State<AppState>,
    ValidatedJson(payload): ValidatedJson<SetupAccountRequest>,
) -> Result<impl IntoResponse, ApiError> {
    state.account_service.setup_account(payload).await?;
    Ok(Json(json!({ "message": "Thiết lập tài khoản thành công" })))
}

/// Đặt lại mật khẩu (token RESET-PASSWORD) từ link mail quên mật khẩu.
pub async fn reset_password(
    State(state): State<AppState>,
    ValidatedJson(payload): ValidatedJson<SetPasswordRequest>,
) -> Result<impl IntoResponse, ApiError> {
    state.account_service.reset_password(payload).await?;
    Ok(Json(json!({ "message": "Đặt lại mật khẩu thành công" })))
}

/// Quên mật khẩu: nhập email để nhận link đặt lại. Luôn trả 200 chung chung (không lộ email).
pub async fn forgot_password(
    State(state): State<AppState>,
    ValidatedJson(payload): ValidatedJson<ForgotPasswordRequest>,
) -> Result<impl IntoResponse, ApiError> {
    state.account_service.forgot_password(payload).await?;
    Ok(Json(json!({
        "message": "Nếu email tồn tại, liên kết đặt lại mật khẩu đã được gửi"
    })))
}

/// Đọc IP client: ưu tiên `X-Forwarded-For` (token đầu) → `X-Real-IP` → địa chỉ peer.
/// Chỉ chấp nhận giá trị parse được thành `IpAddr` để tránh fail cast `::inet` (500).
fn client_ip(headers: &HeaderMap, peer: SocketAddr) -> String {
    if let Some(xff) = headers.get("x-forwarded-for").and_then(|v| v.to_str().ok())
        && let Some(first) = xff.split(',').next()
        && first.trim().parse::<IpAddr>().is_ok()
    {
        return first.trim().to_string();
    }
    if let Some(xrip) = headers.get("x-real-ip").and_then(|v| v.to_str().ok())
        && xrip.trim().parse::<IpAddr>().is_ok()
    {
        return xrip.trim().to_string();
    }
    peer.ip().to_string()
}

fn user_agent(headers: &HeaderMap) -> Option<String> {
    headers
        .get(axum::http::header::USER_AGENT)
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string())
}

/// Cookie `session` với thuộc tính dùng chung. `max_age = None` (kèm value rỗng) dùng để xóa cookie —
/// PHẢI khớp `path`/`domain` với cookie lúc login thì trình duyệt mới gỡ.
fn session_cookie(
    value: String,
    secure: bool,
    domain: Option<&str>,
    max_age: Option<time::Duration>,
) -> Cookie<'static> {
    let mut builder = Cookie::build(("session", value))
        .http_only(true)
        .secure(secure)
        .same_site(SameSite::Lax)
        .path("/");

    if let Some(d) = domain {
        builder = builder.domain(d.to_owned());
    }
    if let Some(age) = max_age {
        builder = builder.max_age(age);
    }

    builder.build()
}
