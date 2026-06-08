use futures_lite::stream::StreamExt;
use lapin::options::{
    BasicAckOptions, BasicConsumeOptions, BasicNackOptions, BasicQosOptions, QueueDeclareOptions,
};
use lapin::types::FieldTable;
use lapin::{Connection, ConnectionProperties};
use shared::messaging::EmailJob;

mod config;
mod email;

use config::WorkerConfig;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();
    tracing_subscriber::fmt::init();

    let cfg = WorkerConfig::from_env()?;
    let http = reqwest::Client::new();

    let conn = Connection::connect(&cfg.rabbitmq_url, ConnectionProperties::default()).await?;
    let channel = conn.create_channel().await?;
    channel.basic_qos(10, BasicQosOptions::default()).await?;
    channel
        .queue_declare(
            cfg.rabbitmq_email_queue.as_str().into(),
            QueueDeclareOptions {
                durable: true,
                ..Default::default()
            },
            FieldTable::default(),
        )
        .await?;

    let mut consumer = channel
        .basic_consume(
            cfg.rabbitmq_email_queue.as_str().into(),
            "email-worker".into(),
            BasicConsumeOptions::default(),
            FieldTable::default(),
        )
        .await?;

    tracing::info!(
        "Worker email đang chạy, chờ job từ queue '{}'",
        cfg.rabbitmq_email_queue
    );

    let consume = async {
        while let Some(delivery) = consumer.next().await {
            let delivery = match delivery {
                Ok(d) => d,
                Err(e) => {
                    tracing::error!("lỗi nhận delivery: {e}");
                    continue;
                }
            };

            let job = match serde_json::from_slice::<EmailJob>(&delivery.data) {
                Ok(job) => job,
                Err(err) => {
                    tracing::error!("payload EmailJob hỏng, bỏ qua: {err:#}");
                    let _ = delivery
                        .nack(BasicNackOptions {
                            requeue: false,
                            ..Default::default()
                        })
                        .await;
                    continue;
                }
            };

            // Gửi mail theo loại job; trả (mô tả, kết quả) để ack/nack chung.
            let (to, result) = match &job {
                EmailJob::SetPassword(j) => (
                    j.to.clone(),
                    email::gmail::send_set_password_email(&cfg, &http, j).await,
                ),
                EmailJob::ResetPassword(j) => (
                    j.to.clone(),
                    email::gmail::send_reset_password_email(&cfg, &http, j).await,
                ),
            };

            match result {
                Ok(()) => {
                    tracing::info!("đã gửi email tới {to}");
                    let _ = delivery.ack(BasicAckOptions::default()).await;
                }
                Err(err) => {
                    tracing::error!("gửi mail tới {to} thất bại: {err:#}");
                    let _ = delivery
                        .nack(BasicNackOptions {
                            requeue: false,
                            ..Default::default()
                        })
                        .await;
                }
            }
        }
    };

    // Graceful shutdown khi Ctrl-C.
    tokio::select! {
        _ = consume => {},
        _ = tokio::signal::ctrl_c() => {
            tracing::info!("Nhận tín hiệu dừng, thoát worker.");
        }
    }

    Ok(())
}
