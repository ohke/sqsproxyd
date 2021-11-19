use anyhow::Result;
use async_trait::async_trait;
use aws_sdk_sqs::model::Message;
use aws_sdk_sqs::Client;

#[async_trait]
pub trait Sqs {
    async fn receive_messages(&self) -> Result<Option<Vec<Message>>>;
    async fn send_message(&self, message_body: String) -> Result<()>;
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
        let output = self
            .client
            .receive_message()
            .queue_url(&self.url)
            .send()
            .await?;
        Ok(output.messages)
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
