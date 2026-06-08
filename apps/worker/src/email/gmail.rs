use anyhow::Context;
use lettre::message::header::ContentType;
use lettre::transport::smtp::authentication::{Credentials, Mechanism};
use lettre::{AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor};
use serde::Deserialize;
use shared::messaging::{ResetPasswordEmail, SetPasswordEmail};

use crate::config::WorkerConfig;

#[derive(Deserialize)]
struct TokenResponse {
    access_token: String,
}

/// Đổi refresh token → access token qua Google OAuth2 token endpoint.
async fn fetch_access_token(cfg: &WorkerConfig, http: &reqwest::Client) -> anyhow::Result<String> {
    let form = serde_urlencoded::to_string([
        ("client_id", cfg.gmail_client_id.as_str()),
        ("client_secret", cfg.gmail_client_secret.as_str()),
        ("refresh_token", cfg.gmail_refresh_token.as_str()),
        ("grant_type", "refresh_token"),
    ])?;

    let resp = http
        .post("https://oauth2.googleapis.com/token")
        .header(
            reqwest::header::CONTENT_TYPE,
            "application/x-www-form-urlencoded",
        )
        .body(form)
        .send()
        .await?
        .error_for_status()
        .context("Google token endpoint trả lỗi")?
        .json::<TokenResponse>()
        .await?;
    Ok(resp.access_token)
}

/// Gửi 1 email HTML qua Gmail SMTP + XOAUTH2 (access token đặt ở ô password).
async fn send_html(
    cfg: &WorkerConfig,
    http: &reqwest::Client,
    to: &str,
    subject: &str,
    body: String,
) -> anyhow::Result<()> {
    let access_token = fetch_access_token(cfg, http).await?;
    let creds = Credentials::new(cfg.gmail_sender.clone(), access_token);

    let mailer = AsyncSmtpTransport::<Tokio1Executor>::starttls_relay("smtp.gmail.com")?
        .port(587)
        .authentication(vec![Mechanism::Xoauth2])
        .credentials(creds)
        .build();

    let email = Message::builder()
        .from(cfg.gmail_sender.parse()?)
        .to(to.parse()?)
        .subject(subject)
        .header(ContentType::TEXT_HTML)
        .body(body)?;

    mailer.send(email).await?;
    Ok(())
}

/// Email "thiết lập tài khoản" (admin tạo tài khoản → user đặt tên đăng nhập + mật khẩu).
pub async fn send_set_password_email(
    cfg: &WorkerConfig,
    http: &reqwest::Client,
    e: &SetPasswordEmail,
) -> anyhow::Result<()> {
    let body = format!(
        "<p>Chào bạn,</p>\
         <p>Tài khoản của bạn vừa được tạo. Nhấn vào liên kết dưới đây để thiết lập tên đăng nhập và mật khẩu:</p>\
         <p><a href=\"{url}\">Thiết lập tài khoản</a></p>\
         <p>Liên kết có hiệu lực trong {hours} giờ.</p>",
        url = e.set_password_url,
        hours = e.expires_in_hours,
    );
    send_html(
        cfg,
        http,
        &e.to,
        "Thiết lập tài khoản của bạn",
        body,
    )
    .await
}

/// Email "đặt lại mật khẩu" (user quên mật khẩu).
pub async fn send_reset_password_email(
    cfg: &WorkerConfig,
    http: &reqwest::Client,
    e: &ResetPasswordEmail,
) -> anyhow::Result<()> {
    let body = format!(
        "<p>Chào bạn,</p>\
         <p>Bạn (hoặc ai đó) đã yêu cầu đặt lại mật khẩu cho tài khoản này. Nhấn vào liên kết dưới đây để đặt mật khẩu mới:</p>\
         <p><a href=\"{url}\">Đặt lại mật khẩu</a></p>\
         <p>Liên kết có hiệu lực trong {hours} giờ. Nếu không phải bạn yêu cầu, hãy bỏ qua email này.</p>",
        url = e.reset_password_url,
        hours = e.expires_in_hours,
    );
    send_html(
        cfg,
        http,
        &e.to,
        "Đặt lại mật khẩu tài khoản của bạn",
        body,
    )
    .await
}
