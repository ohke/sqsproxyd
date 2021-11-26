use serde::{Deserialize, Serialize};

#[derive(Debug)]
pub struct Message {
    pub body: MessageBody,
    pub receipt_handle: String,
}

impl From<aws_sdk_sqs::model::Message> for Message {
    fn from(item: aws_sdk_sqs::model::Message) -> Self {
        Message {
            body: serde_json::from_str(&item.body.unwrap()).unwrap(),
            receipt_handle: item.receipt_handle.unwrap(),
        }
    }
}

#[derive(Deserialize, Serialize, PartialEq, Debug)]
pub struct MessageBody {
    pub path: String,
    pub data: String,
    pub context: Option<String>,
}
