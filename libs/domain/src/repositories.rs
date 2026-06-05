use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::entities::{NewSession, Session, User};
use crate::errors::DomainError;

pub trait UserRepository: Send + Sync {
    // async fn find_by_email(&self, email: &str) -> Result<Option<User>, DomainError>;
    // async fn find_by_id(&self, id: &str) -> Result<Option<User>, DomainError>;
    // async fn create(&self, user: &User) -> Result<User, DomainError>;

    fn find_by_email(
        &self,
        email: &str,
    ) -> impl Future<Output = Result<Option<User>, DomainError>> + Send;

    fn find_by_id(
        &self,
        id: uuid::Uuid,
    ) -> impl Future<Output = Result<Option<User>, DomainError>> + Send;

    // fn create(&self, user: &User) -> impl Future<Output = Result<User, DomainError>> + Send;
}

pub trait UserSessionRepository: Send + Sync {
    fn create(
        &self,
        new_session: NewSession,
    ) -> impl Future<Output = Result<Session, DomainError>> + Send;

    fn find_by_token_hash(
        &self,
        token_hash: &str,
    ) -> impl Future<Output = Result<Option<Session>, DomainError>> + Send;

    /// Thu hồi một phiên cụ thể (logout). No-op nếu phiên đã thu hồi trước đó.
    fn revoke(
        &self,
        id: Uuid,
        reason: &str,
    ) -> impl Future<Output = Result<(), DomainError>> + Send;

    /// Thu hồi tất cả phiên còn hiệu lực của một user (logout mọi thiết bị).
    fn revoke_all_for_user(
        &self,
        user_id: Uuid,
        reason: &str,
    ) -> impl Future<Output = Result<(), DomainError>> + Send;

    /// Cập nhật `expires_at` cho phiên (sliding). No-op nếu phiên đã thu hồi.
    fn touch_expires(
        &self,
        id: Uuid,
        expires_at: DateTime<Utc>,
    ) -> impl Future<Output = Result<(), DomainError>> + Send;
}
