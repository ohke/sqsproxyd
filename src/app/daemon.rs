use crate::{AwsSqs, WebhookImpl};
use anyhow::Result;
use std::borrow::Borrow;
use tokio::{
    sync::{broadcast, mpsc},
    time::{sleep, Duration},
};
use tracing::{info, warn};

use crate::domain::config::Config;
use crate::domain::message::Message;
use crate::infra::sqs::Sqs;
use crate::infra::webhook::Webhook;

pub struct Daemon {
    config: Config,
    sqs: Box<dyn Sqs + Send + Sync>,
    webhook: Box<dyn Webhook + Send + Sync>,
}

impl Daemon {
    pub async fn new(config: Config) -> Self {
        Daemon {
            config: config.clone(),
            sqs: Box::new(
                AwsSqs::new(config.sqs_url.to_string(), config.max_number_of_messages).await,
            ),
            webhook: Box::new(WebhookImpl::new(config.clone())),
        }
    }

    pub async fn run(
        self,
        mut shutdown_rx: broadcast::Receiver<()>,
        _heartbeat_tx: mpsc::Sender<()>,
    ) -> Result<()> {
        info!("{:?}", self.config);

        if let Some(_url) = &self.config.webhook_health_check_url {
            tokio::select! {
                result = Self::health_check(self.webhook.borrow()) => result.unwrap(),
                _ = shutdown_rx.recv() => return Ok(()),
            }
        }

        // create workers
        let (tx, rx) = async_channel::bounded::<Message>(self.config.worker_concurrency);
        let (worker_shutdown_tx, _) = broadcast::channel(1);
        let (worker_heartbeat_tx, mut worker_heartbeat_rx) = mpsc::channel::<()>(1);

        for _ in 0..self.config.worker_concurrency {
            let config = self.config.clone();
            let sqs = Box::new(
                AwsSqs::new(config.sqs_url.to_string(), config.max_number_of_messages).await,
            );
            let webhook = Box::new(WebhookImpl::new(self.config.clone()));
            let output_sqs: Option<Box<dyn Sqs + Send + Sync>> = match &self.config.output_sqs_url {
                None => None,
                Some(u) => Some(Box::new(
                    AwsSqs::new(u.to_string(), self.config.max_number_of_messages).await,
                )),
            };
            let rx = rx.clone();
            let shutdown_rx = worker_shutdown_tx.subscribe();
            let heartbeat_tx = worker_heartbeat_tx.clone();

            tokio::spawn(async move {
                Self::poll_process(sqs, webhook, output_sqs, rx, shutdown_rx, heartbeat_tx).await
            });
        }

        drop(worker_heartbeat_tx);

        // receive SQS message
        loop {
            tokio::select! {
                result = self.poll() => {
                    let messages = result.unwrap();
                    if let Some(messages) = messages {
                        if messages.is_empty() {
                            self.sleep().await;
                        } else {
                            for message in messages {
                                tx.send(message).await?;
                            }
                        }
                    } else {
                        self.sleep().await;
                    }
                }
                _ = shutdown_rx.recv() => {
                    worker_shutdown_tx.send(()).unwrap();
                    let _ = worker_heartbeat_rx.recv().await;
                    return Ok(());
                }
            }
        }
    }

    async fn health_check(webhook: &'_ (dyn Webhook + Send + Sync)) -> Result<()> {
        loop {
            if webhook.get().await.is_ok() {
                break;
            }
        }

        Ok(())
    }

    async fn poll(&self) -> Result<Option<Vec<Message>>> {
        self.sqs.receive_messages().await
    }

    async fn poll_process(
        sqs: Box<dyn Sqs + Send + Sync>,
        webhook: Box<dyn Webhook + Send + Sync>,
        output_sqs: Option<Box<dyn Sqs + Send + Sync>>,
        rx: async_channel::Receiver<Message>,
        mut shutdown_rx: broadcast::Receiver<()>,
        _heartbeat_tx: mpsc::Sender<()>,
    ) -> Result<()> {
        loop {
            tokio::select! {
                result = rx.recv() => {
                    let message = result.unwrap();
                    info!("{:?}", message);

                    if message.check_hash() {
                        let _ = Self::process_message(message, sqs.borrow(), webhook.borrow(), &output_sqs).await;
                    } else {
                        warn!("Mismatch message MD5 digest.");
                    }
                }
                _ = shutdown_rx.recv() => return Ok(()),
            }
        }
    }

    async fn process_message(
        message: Message,
        sqs: &'_ (dyn Sqs + Send + Sync),
        webhook: &'_ (dyn Webhook + Send + Sync),
        output_sqs: &Option<Box<dyn Sqs + Send + Sync>>,
    ) -> Result<()> {
        let (is_successed, res) = webhook
            .post(message.body.clone(), &message.message_id)
            .await?;
        if !is_successed {
            info!("Not succeeded: {:?}", &res);
            return Ok(());
        }

        if output_sqs.is_some() {
            output_sqs.as_ref().unwrap().send_message(res).await?;
        }

        sqs.delete_message(message.receipt_handle).await?;

        Ok(())
    }

    async fn sleep(&self) {
        info!("sleep");
        sleep(Duration::from_secs(self.config.sleep_seconds)).await;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::message::*;
    use crate::infra::sqs::*;
    use crate::infra::webhook::*;
    use anyhow::anyhow;
    use mockall::predicate::*;
    use std::borrow::Borrow;

    #[tokio::test]
    async fn test_process_message_with_output() {
        dotenv::from_filename("env/test.env").expect("Not found env file.");

        let mut sqs = MockSqs::new();
        sqs.expect_send_message().times(0).returning(|_| Ok(()));
        sqs.expect_delete_message()
            .with(eq("receipt_handle".to_string()))
            .times(1)
            .returning(|_| Ok(()));
        let sqs: Box<dyn Sqs + Send + Sync> = Box::new(sqs);

        let mut webhook = MockWebhook::new();
        webhook.expect_post().times(1).returning(|_, message_id| {
            assert_eq!(message_id, "message_id");
            Ok((true, "result".to_string()))
        });
        let webhook: Box<dyn Webhook + Send + Sync> = Box::new(webhook);

        let mut output_sqs = MockSqs::new();
        output_sqs.expect_receive_messages().times(0);
        output_sqs
            .expect_send_message()
            .with(eq("result".to_string()))
            .times(1)
            .returning(|_| Ok(()));
        output_sqs.expect_delete_message().times(0);
        let output_sqs: Option<Box<dyn Sqs + Send + Sync>> = Some(Box::new(output_sqs));

        let message = Message {
            receipt_handle: "receipt_handle".to_string(),
            body: "{\"key1\": 1}".to_string(),
            md5_of_body: "dummy".to_string(),
            message_id: "message_id".to_string(),
        };

        Daemon::process_message(message, sqs.borrow(), webhook.borrow(), &output_sqs)
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn test_process_message_without_output() {
        dotenv::from_filename("env/test.env").expect("Not found env file.");

        let mut sqs = MockSqs::new();
        sqs.expect_send_message().times(0).returning(|_| Ok(()));
        sqs.expect_delete_message()
            .with(eq("receipt_handle".to_string()))
            .times(1)
            .returning(|_| Ok(()));
        let sqs: Box<dyn Sqs + Send + Sync> = Box::new(sqs);

        let mut webhook = MockWebhook::new();
        webhook.expect_post().times(1).returning(|_, message_id| {
            assert_eq!(message_id, "message_id");
            Ok((true, "result".to_string()))
        });
        let webhook: Box<dyn Webhook + Send + Sync> = Box::new(webhook);

        let output_sqs = None;

        let message = Message {
            receipt_handle: "receipt_handle".to_string(),
            body: "{\"key1\": 1}".to_string(),
            md5_of_body: "dummy".to_string(),
            message_id: "message_id".to_string(),
        };

        Daemon::process_message(message, sqs.borrow(), webhook.borrow(), &output_sqs)
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn test_process_message_if_failed_not_deleted() {
        let mut sqs = MockSqs::new();
        sqs.expect_delete_message().times(0);
        let sqs: Box<dyn Sqs + Send + Sync> = Box::new(sqs);

        let mut webhook = MockWebhook::new();
        webhook.expect_post().times(1).returning(|_, message_id| {
            assert_eq!(message_id, "message_id");
            Ok((false, "result".to_string()))
        });
        let webhook: Box<dyn Webhook + Send + Sync> = Box::new(webhook);

        let mut output_sqs = MockSqs::new();
        output_sqs.expect_receive_messages().times(0);
        output_sqs.expect_send_message().times(0);
        output_sqs.expect_delete_message().times(0);
        let output_sqs: Option<Box<dyn Sqs + Send + Sync>> = Some(Box::new(output_sqs));

        let message = Message {
            receipt_handle: "receipt_handle".to_string(),
            body: "{\"key1\": 1}".to_string(),
            md5_of_body: "dummy".to_string(),
            message_id: "message_id".to_string(),
        };

        Daemon::process_message(message, sqs.borrow(), webhook.borrow(), &output_sqs)
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn test_health_check() {
        let mut webhook = MockWebhook::new();
        webhook
            .expect_get()
            .times(3)
            .returning(|| Err(anyhow!("Error")))
            .times(1)
            .returning(|| Ok(()));
        let webhook: Box<dyn Webhook + Send + Sync> = Box::new(webhook);

        Daemon::health_check(webhook.borrow()).await.unwrap();
    }
}
