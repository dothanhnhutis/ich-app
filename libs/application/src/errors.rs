use domain::errors::DomainError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Unauthorized: {0}")]
    Unauthorized(String),

    #[error("Internal error: {0}")]
    Internal(String),
}

impl From<DomainError> for AppError {
    fn from(err: DomainError) -> Self {
        match err {
            DomainError::NotFound(msg) => AppError::NotFound(msg),
            DomainError::InvalidCredentials => {
                AppError::Unauthorized("Invalid credentials".into())
            }
            DomainError::Validation(msg) => AppError::Validation(msg),
            DomainError::AlreadyExists(msg) => AppError::Validation(msg),
            DomainError::Internal(msg) => AppError::Internal(msg),
        }
    }
}
