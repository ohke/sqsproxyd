use anyhow::Result;
use tokio::{
    sync::{broadcast, mpsc},
    time::{sleep, Duration},
};
use tracing::info;

use crate::domain::config::Config;
use crate::domain::message::{Message, MessageBody};
use crate::infra::sqs::Sqs;
use crate::infra::webhook::Webhook;

pub struct Daemon {
    config: Config,
    sqs: Box<dyn Sqs + Send + Sync>,
    webhook: Box<dyn Webhook + Send + Sync>,
    output_sqs: Option<Box<dyn Sqs + Send + Sync>>,
}

impl Daemon {
    pub fn new(
        config: Config,
        sqs: Box<dyn Sqs + Send + Sync>,
        webhook: Box<dyn Webhook + Send + Sync>,
        output_sqs: Option<Box<dyn Sqs + Send + Sync>>,
    ) -> Self {
        Daemon {
            config,
            sqs,
            webhook,
            output_sqs,
        }
    }

    pub async fn run(
        self,
        mut shutdown_rx: broadcast::Receiver<()>,
        _heartbeat_tx: mpsc::Sender<()>,
    ) -> Result<()> {
        info!("{:?}", self.config);

        tokio::select! {
            result = self.init() => result.unwrap(),
            _ = shutdown_rx.recv() => {
                info!("Shutdown (init) .");
                return Ok(());
            }
        }

        loop {
            tokio::select! {
                result = self.poll() => {
                    let messages = result.unwrap();
                    if let Some(messages) = messages {
                        if messages.is_empty() {
                            self.sleep().await;
                        } else {
                            self.process(messages).await?;
                        }
                    } else {
                        self.sleep().await;
                    }
                }
                _ = shutdown_rx.recv() => {
                    info!("Shutdown.");
                    return Ok(());
                }
            }
        }
    }

    async fn init(&self) -> Result<()> {
        if let Some(path) = &self.config.webhook_health_check_path {
            loop {
                if self.webhook.get(path).await.is_ok() {
                    break;
                }
            }
        }

        Ok(())
    }

    async fn poll(&self) -> Result<Option<Vec<Message>>> {
        self.sqs.receive_messages().await
    }

    async fn process(&self, messages: Vec<Message>) -> Result<()> {
        for message in messages {
            info!("{:?}", message);

            let (is_successed, res) = self
                .webhook
                .post(&message.body.path, message.body.data.clone())
                .await?;
            if !is_successed {
                info!("Not succeeded: {:?}", &res);
                continue;
            }

            if self.output_sqs.is_some() {
                self.output_sqs
                    .as_ref()
                    .unwrap()
                    .send_message(MessageBody {
                        path: message.body.path.clone(),
                        data: res,
                        context: message.body.context.clone(),
                    })
                    .await?;
            }

            self.sqs.delete_message(message.receipt_handle).await?;
        }

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
    use crate::domain::arg::*;
    use crate::domain::message::*;
    use crate::infra::sqs::*;
    use crate::infra::webhook::*;
    use mockall::predicate::*;

    #[tokio::test]
    async fn test_process_1_message() {
        dotenv::from_filename("env/test.env").expect("Not found env file.");

        let mut sqs = MockSqs::new();
        sqs.expect_receive_messages().times(1).returning(|| {
            Ok(Some(vec![Message {
                receipt_handle: "receipt_handle".to_string(),
                body: MessageBody {
                    path: "/hoge".to_string(),
                    data: "{\"key1\": 1}".to_string(),
                    context: Some("".to_string()),
                },
            }]))
        });
        sqs.expect_send_message().times(0).returning(|_| Ok(()));
        sqs.expect_delete_message()
            .with(eq("receipt_handle".to_string()))
            .times(1)
            .returning(|_| Ok(()));

        let mut webhook = MockWebhook::new();
        webhook
            .expect_post()
            .times(1)
            .returning(|_, _| Ok((true, "result".to_string())));

        let mut output_sqs = MockSqs::new();
        output_sqs.expect_receive_messages().times(0);
        output_sqs
            .expect_send_message()
            .with(eq(MessageBody {
                path: "/hoge".to_string(),
                data: "result".to_string(),
                context: Some("".to_string()),
            }))
            .times(1)
            .returning(|_| Ok(()));
        output_sqs.expect_delete_message().times(0);

        let config = Config::new(Arg::new_empty()).unwrap();

        let daemon = Daemon::new(
            config,
            Box::new(sqs),
            Box::new(webhook),
            Some(Box::new(output_sqs)),
        );

        let messages = daemon.poll().await.unwrap().unwrap();
        assert_eq!(messages.len(), 1);
        assert_eq!(
            messages[0],
            Message {
                receipt_handle: "receipt_handle".to_string(),
                body: MessageBody {
                    path: "/hoge".to_string(),
                    data: "{\"key1\": 1}".to_string(),
                    context: Some("".to_string()),
                },
            }
        );

        daemon
            .process(vec![Message {
                body: MessageBody {
                    path: "/hoge".to_string(),
                    data: "{\"key1\": 1}".to_string(),
                    context: Some("".to_string()),
                },
                receipt_handle: "receipt_handle".to_string(),
            }])
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn test_process_1_failed_message() {
        dotenv::from_filename("env/test.env").expect("Not found env file.");

        let mut sqs = MockSqs::new();
        sqs.expect_receive_messages().times(1).returning(|| {
            Ok(Some(vec![Message {
                receipt_handle: "receipt_handle".to_string(),
                body: MessageBody {
                    path: "/hoge".to_string(),
                    data: "{\"key1\": 1}".to_string(),
                    context: Some("".to_string()),
                },
            }]))
        });
        sqs.expect_send_message().times(0).returning(|_| Ok(()));
        sqs.expect_delete_message().times(0);

        let mut webhook = MockWebhook::new();
        webhook
            .expect_post()
            .times(1)
            .returning(|_, _| Ok((false, "result".to_string())));

        let mut output_sqs = MockSqs::new();
        output_sqs.expect_receive_messages().times(0);
        output_sqs.expect_send_message().times(0);
        output_sqs.expect_delete_message().times(0);

        let config = Config::new(Arg::new_empty()).unwrap();

        let daemon = Daemon::new(
            config,
            Box::new(sqs),
            Box::new(webhook),
            Some(Box::new(output_sqs)),
        );

        let messages = daemon.poll().await.unwrap().unwrap();
        assert_eq!(messages.len(), 1);
        assert_eq!(
            messages[0],
            Message {
                receipt_handle: "receipt_handle".to_string(),
                body: MessageBody {
                    path: "/hoge".to_string(),
                    data: "{\"key1\": 1}".to_string(),
                    context: Some("".to_string()),
                },
            }
        );

        daemon
            .process(vec![Message {
                receipt_handle: "receipt_handle".to_string(),
                body: MessageBody {
                    path: "/hoge".to_string(),
                    data: "{\"key1\": 1}".to_string(),
                    context: Some("".to_string()),
                },
            }])
            .await
            .unwrap();
    }
}
