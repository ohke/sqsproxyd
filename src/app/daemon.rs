use anyhow::Result;
use tokio::time::{sleep, Duration};

use crate::domain::config::Config;
use crate::infra::sqs::Sqs;
use crate::infra::webhook::Webhook;

pub struct Daemon {
    config: Config,
    sqs: Box<dyn Sqs>,
    webhook: Box<dyn Webhook>,
}

impl Daemon {
    pub fn new(config: Config, sqs: Box<dyn Sqs>, webhook: Box<dyn Webhook>) -> Self {
        Daemon {
            config,
            sqs,
            webhook,
        }
    }

    pub async fn run(self) -> Result<()> {
        println!("{:?}", self.config);

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
                println!("{:?}", message);
                let res = self
                    .webhook
                    .post(message.body.path, message.body.data)
                    .await?;
                println!("{}", res);
                self.sqs.delete_message(message.receipt_handle).await?;
            }
            Ok(true)
        } else {
            Ok(false)
        }
    }

    async fn sleep(&self) {
        // TODO: logger
        println!("wait");
        sleep(Duration::from_millis(1000)).await;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::message::*;
    use crate::infra::sqs::*;
    use crate::infra::webhook::*;

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
        sqs.expect_delete_message().times(1).returning(|_| Ok(()));

        let mut webhook = MockWebhook::new();
        webhook
            .expect_post()
            .times(1)
            .returning(|_, _| Ok("".to_string()));

        let config = Config::new().unwrap();

        let daemon = Daemon::new(config, Box::new(sqs), Box::new(webhook));
        assert!(daemon.process().await.unwrap());
    }
}
