use crate::entities::User;
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
