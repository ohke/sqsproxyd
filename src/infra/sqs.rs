use anyhow::Result;
use async_trait::async_trait;
use aws_sdk_sqs::Client;

use crate::domain::message::Message;

use crate::infra::aws::load_aws_config;
use crate::Config;
#[cfg(test)]
use mockall::automock;

#[cfg_attr(test, automock)]
#[async_trait]
pub trait Sqs {
    async fn receive_messages(&self) -> Result<Option<Vec<Message>>>;
    async fn send_message(&self, body: String) -> Result<()>;
    async fn delete_message(&self, receipt_handle: String) -> Result<()>;
}

pub struct AwsSqs {
    client: Client,
    url: String,
    max_number_of_messages: usize,
}

impl AwsSqs {
    pub async fn new(url: String, config: &Config) -> Self {
        AwsSqs {
            client: Client::new(&load_aws_config(config).await),
            url,
            max_number_of_messages: config.max_number_of_messages,
        }
    }
}

#[async_trait]
impl Sqs for AwsSqs {
    async fn receive_messages(&self) -> Result<Option<Vec<Message>>> {
        match self
            .client
            .receive_message()
            .queue_url(&self.url)
            .max_number_of_messages(TryFrom::try_from(self.max_number_of_messages).unwrap())
            .send()
            .await?
            .messages
        {
            None => Ok(None),
            Some(messages) => Ok(Some(messages.into_iter().map(Message::from).collect())),
        }
    }

    async fn send_message(&self, body: String) -> Result<()> {
        self.client
            .send_message()
            .queue_url(&self.url)
            .message_body(serde_json::to_string(&body)?)
            .send()
            .await?;
        Ok(())
    }

    async fn delete_message(&self, receipt_handle: String) -> Result<()> {
        self.client
            .delete_message()
            .queue_url(&self.url)
            .receipt_handle(&receipt_handle)
            .send()
            .await?;
        Ok(())
    }
}
