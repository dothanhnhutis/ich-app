use serde::Deserialize;
use uuid::Uuid;

#[derive(Debug, Deserialize)]
pub struct CreateSessionRequest {
    pub user_id: Uuid,

    pub app_version: Option<String>,

    pub platform: Option<String>,

    pub device_type: String,

    pub device_name: Option<String>,

    pub device_id: Option<String>,
}
