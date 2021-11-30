use anyhow::Result;
use tokio::time::{sleep, Duration};
use tracing::info;

use crate::domain::config::Config;
use crate::domain::message::MessageBody;
use crate::infra::sqs::Sqs;
use crate::infra::webhook::Webhook;

pub struct Daemon {
    config: Config,
    sqs: Box<dyn Sqs>,
    webhook: Box<dyn Webhook>,
    output_sqs: Option<Box<dyn Sqs>>,
}

impl Daemon {
    pub fn new(
        config: Config,
        sqs: Box<dyn Sqs>,
        webhook: Box<dyn Webhook>,
        output_sqs: Option<Box<dyn Sqs>>,
    ) -> Self {
        Daemon {
            config,
            sqs,
            webhook,
            output_sqs,
        }
    }

    pub async fn run(self) -> Result<()> {
        info!("{:?}", self.config);

        loop {
            let has_messages = self.process().await?;
            if !has_messages {
                self.sleep().await;
            }
        }
    }

    async fn process(&self) -> Result<bool> {
        if let Some(messages) = self.sqs.receive_messages().await? {
            if messages.is_empty() {
                return Ok(false);
            }
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
            Ok(true)
        } else {
            Ok(false)
        }
    }

    async fn sleep(&self) {
        info!("wait");
        sleep(Duration::from_millis(1000)).await;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
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

        let config = Config::new().unwrap();

        let daemon = Daemon::new(
            config,
            Box::new(sqs),
            Box::new(webhook),
            Some(Box::new(output_sqs)),
        );
        assert!(daemon.process().await.unwrap());
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

        let config = Config::new().unwrap();

        let daemon = Daemon::new(
            config,
            Box::new(sqs),
            Box::new(webhook),
            Some(Box::new(output_sqs)),
        );
        assert!(daemon.process().await.unwrap());
    }
}
