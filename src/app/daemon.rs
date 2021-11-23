use anyhow::Result;
use tokio::time::{sleep, Duration};

use crate::domain::config::Config;
use crate::domain::message::Message;
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
            for m in messages {
                let message: Message = serde_json::from_str(&m.body.unwrap())?;
                println!("{:?}", message);
                let res = self.webhook.post(message.path, message.data).await?;
                println!("{}", res);
                self.sqs.delete_message(m.receipt_handle.unwrap()).await?;
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
