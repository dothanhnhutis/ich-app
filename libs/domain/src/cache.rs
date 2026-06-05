use uuid::Uuid;

use crate::entities::CachedSession;
use crate::errors::DomainError;

/// Port cache phiên (outbound). Adapter (vd Redis) hiện thực ở tầng infrastructure.
pub trait SessionCache: Send + Sync {
    fn get(
        &self,
        token_hash: &str,
    ) -> impl Future<Output = Result<Option<CachedSession>, DomainError>> + Send;

    fn put(
        &self,
        token_hash: &str,
        entry: &CachedSession,
        ttl_secs: i64,
    ) -> impl Future<Output = Result<(), DomainError>> + Send;

    fn remove(&self, token_hash: &str) -> impl Future<Output = Result<(), DomainError>> + Send;

    fn remove_all_for_user(
        &self,
        user_id: Uuid,
    ) -> impl Future<Output = Result<(), DomainError>> + Send;
}
