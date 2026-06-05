use chrono::{DateTime, Utc};
use domain::entities::User;
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct UserResponse {
    pub id: String,
    pub email: String,
    pub username: String,
    pub status: String,
    pub created_at: DateTime<Utc>,
}

impl From<User> for UserResponse {
    fn from(u: User) -> Self {
        Self {
            id: u.id.to_string(),
            email: u.email,
            username: u.username,
            status: u.status.as_str().to_string(),
            created_at: u.created_at,
        }
    }
}
