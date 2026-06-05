use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub id: Uuid,
    pub user_id: Uuid,
    pub token_hash: String,
    pub device_id: Option<String>,
    pub device_name: Option<String>,
    pub device_type: String,
    pub platform: Option<String>,
    pub app_version: Option<String>,
    pub user_agent: Option<String>,
    pub ip_address: Option<String>,
    pub revoked_at: Option<DateTime<Utc>>,
    pub revoke_reason: Option<String>,
    pub expires_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Dữ liệu cần thiết để tạo mới một Session.
/// Chỉ chứa các field do caller cung cấp — `id`, `created_at`, `updated_at`
/// do DB tự sinh; `revoked_at`/`revoke_reason` mặc định NULL.
#[derive(Debug, Clone)]
pub struct NewSession {
    pub user_id: Uuid,
    pub token_hash: String,
    pub device_type: String,
    pub device_id: Option<String>,
    pub device_name: Option<String>,
    pub platform: Option<String>,
    pub app_version: Option<String>,
    pub user_agent: Option<String>,
    pub ip_address: Option<String>,
    pub expires_at: DateTime<Utc>,
}
