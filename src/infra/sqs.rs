use anyhow::Result;
use async_trait::async_trait;
use aws_sdk_sqs::Client;

use crate::domain::message::Message;

#[cfg(test)]
use mockall::automock;

#[cfg_attr(test, automock)]
#[async_trait]
pub trait Sqs {
    async fn receive_messages(&self) -> Result<Option<Vec<Message>>>;
    async fn send_message(&self, message_body: String) -> Result<()>;
    async fn delete_message(&self, receipt_handle: String) -> Result<()>;
}

pub struct AwsSqs {
    client: Client,
    url: String,
}

impl AwsSqs {
    pub async fn new(url: String) -> Self {
        AwsSqs {
            client: Client::new(&aws_config::load_from_env().await),
            url: url.to_string(),
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
            .send()
            .await?
            .messages
        {
            None => Ok(None),
            Some(messages) => Ok(Some(messages.into_iter().map(Message::from).collect())),
        }
    }

    async fn send_message(&self, message_body: String) -> Result<()> {
        self.client
            .send_message()
            .queue_url(&self.url)
            .message_body(message_body)
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

// #[cfg(test)]
// mod tests {
//     use super::*;
//
//     struct MockSqs {
//
//     }
//
//     impl Sqs for MockSqs {
//         fn receive_messages() -> Result<Vec<Message>> {
//             todo!()
//         }
//         fn send_message(message: Message) -> Result<()> {
//             todo!()
//         }
//     }
// }
