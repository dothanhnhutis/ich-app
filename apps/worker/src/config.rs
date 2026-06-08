use std::env;

/// Config riêng cho worker — chỉ cần RabbitMQ + Gmail OAuth2 (không cần DB/Redis).
pub struct WorkerConfig {
    pub rabbitmq_url: String,
    pub rabbitmq_email_queue: String,
    pub gmail_client_id: String,
    pub gmail_client_secret: String,
    pub gmail_refresh_token: String,
    pub gmail_sender: String,
}

impl WorkerConfig {
    pub fn from_env() -> anyhow::Result<Self> {
        Ok(Self {
            rabbitmq_url: env::var("RABBITMQ_URL")
                .unwrap_or_else(|_| "amqp://guest:guest@127.0.0.1:5672/%2f".into()),
            rabbitmq_email_queue: env::var("RABBITMQ_EMAIL_QUEUE")
                .unwrap_or_else(|_| "email_jobs".into()),
            gmail_client_id: env::var("GMAIL_CLIENT_ID")
                .map_err(|_| anyhow::anyhow!("GMAIL_CLIENT_ID must be set"))?,
            gmail_client_secret: env::var("GMAIL_CLIENT_SECRET")
                .map_err(|_| anyhow::anyhow!("GMAIL_CLIENT_SECRET must be set"))?,
            gmail_refresh_token: env::var("GMAIL_REFRESH_TOKEN")
                .map_err(|_| anyhow::anyhow!("GMAIL_REFRESH_TOKEN must be set"))?,
            gmail_sender: env::var("GMAIL_SENDER")
                .map_err(|_| anyhow::anyhow!("GMAIL_SENDER must be set"))?,
        })
    }
}
