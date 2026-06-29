mod cache;
mod repositories;

pub use cache::*;
pub use repositories::*;

use crate::errors::AppError;
use shared::messaging::EmailJob;

/// Outbound port: đẩy email job vào hàng chờ (RabbitMQ). Adapter ở tầng infrastructure.
pub trait EmailPublisher: Send + Sync {
    fn publish(&self, job: EmailJob) -> impl Future<Output = Result<(), AppError>> + Send;
}
