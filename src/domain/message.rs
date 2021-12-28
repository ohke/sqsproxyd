#[derive(PartialEq, Debug)]
pub struct Message {
    pub body: String,
    pub receipt_handle: String,
}

impl From<aws_sdk_sqs::model::Message> for Message {
    fn from(item: aws_sdk_sqs::model::Message) -> Self {
        Message {
            body: item.body.unwrap(),
            receipt_handle: item.receipt_handle.unwrap(),
        }
    }
}
