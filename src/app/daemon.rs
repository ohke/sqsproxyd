use anyhow::Result;
use tokio::time::{sleep, Duration};

use crate::domain::config::Config;
use crate::domain::message::Message;
use crate::infra::sqs::Sqs;

pub struct Daemon {
    config: Config,
    sqs: Box<dyn Sqs>,
}

impl Daemon {
    pub fn new(config: Config, sqs: Box<dyn Sqs>) -> Self {
        Daemon { config, sqs }
    }

    pub async fn run(self) -> Result<()> {
        println!("{:?}", self.config);

        loop {
            if let Some(messages) = self.sqs.receive_messages().await? {
                if messages.is_empty() {
                    self.sleep().await;
                }
                for m in messages {
                    let message: Message = serde_json::from_str(&m.body.unwrap())?;
                    println!("{:?}", message);
                    self.sqs.delete_message(m.receipt_handle.unwrap()).await?;
                }
            } else {
                self.sleep().await;
            }
        }
    }

    async fn sleep(&self) {
        // TODO: logger
        println!("wait");
        sleep(Duration::from_millis(1000)).await;
    }
}
