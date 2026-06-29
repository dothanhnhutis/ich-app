use uuid::Uuid;

use crate::errors::AppError;
use domain::entities::CachedSession;

/// Port cache phiên (outbound). Adapter (vd Redis) hiện thực ở tầng infrastructure.
pub trait SessionCache: Send + Sync {
    fn get(
        &self,
        token_hash: &str,
    ) -> impl Future<Output = Result<Option<CachedSession>, AppError>> + Send;

    fn put(
        &self,
        token_hash: &str,
        entry: &CachedSession,
        ttl_secs: i64,
    ) -> impl Future<Output = Result<(), AppError>> + Send;

    fn remove(&self, token_hash: &str) -> impl Future<Output = Result<(), AppError>> + Send;

    fn remove_all_for_user(
        &self,
        user_id: Uuid,
    ) -> impl Future<Output = Result<(), AppError>> + Send;
}
