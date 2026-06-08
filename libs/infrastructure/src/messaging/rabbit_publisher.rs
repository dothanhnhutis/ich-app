use std::sync::Arc;

use application::errors::AppError;
use application::ports::EmailPublisher;
use lapin::options::{BasicPublishOptions, QueueDeclareOptions};
use lapin::types::FieldTable;
use lapin::{BasicProperties, Channel, Connection, ConnectionProperties};
use shared::messaging::EmailJob;

/// Publisher đẩy EmailJob vào RabbitMQ. Giữ `Connection` để channel không bị đóng khi drop.
#[derive(Clone)]
pub struct RabbitEmailPublisher {
    _conn: Arc<Connection>,
    channel: Channel,
    queue: String,
}

impl RabbitEmailPublisher {
    pub async fn connect(url: &str, queue: &str) -> anyhow::Result<Self> {
        let conn = Connection::connect(url, ConnectionProperties::default()).await?;
        let channel = conn.create_channel().await?;
        channel
            .queue_declare(
                queue.into(),
                QueueDeclareOptions {
                    durable: true,
                    ..Default::default()
                },
                FieldTable::default(),
            )
            .await?;
        Ok(Self {
            _conn: Arc::new(conn),
            channel,
            queue: queue.to_string(),
        })
    }
}

impl EmailPublisher for RabbitEmailPublisher {
    async fn publish(&self, job: EmailJob) -> Result<(), AppError> {
        let payload = serde_json::to_vec(&job).map_err(|e| AppError::Internal(e.to_string()))?;
        self.channel
            .basic_publish(
                "".into(),
                self.queue.as_str().into(),
                BasicPublishOptions::default(),
                &payload,
                BasicProperties::default().with_delivery_mode(2), // persistent
            )
            .await
            .map_err(|e| AppError::Internal(e.to_string()))?
            .await
            .map_err(|e| AppError::Internal(e.to_string()))?;
        Ok(())
    }
}
