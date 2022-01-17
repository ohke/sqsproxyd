use anyhow::Result;
use async_trait::async_trait;
use aws_sdk_sqs::{Client, Endpoint};

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
    max_number_of_messages: i32,
}

impl AwsSqs {
    pub async fn new(url: String, config: &Config) -> Self {
        let client = match &config.aws_endpoint {
            None => Client::new(&load_aws_config(config).await),
            Some(aws_endpoint) => {
                let aws_config = load_aws_config(config).await;
                let sqs_config = aws_sdk_sqs::config::Builder::from(&aws_config)
                    .endpoint_resolver(Endpoint::immutable(aws_endpoint.clone()))
                    .build();
                aws_sdk_sqs::Client::from_conf(sqs_config)
            }
        };
        AwsSqs {
            client,
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
            .max_number_of_messages(self.max_number_of_messages)
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
