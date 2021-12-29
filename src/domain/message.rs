use md5;

#[derive(PartialEq, Debug)]
pub struct Message {
    pub body: String,
    pub receipt_handle: String,
    pub md5_of_body: String,
}

impl Message {
    pub fn check_hash(&self) -> bool {
        let digest = md5::compute(&self.body);
        format!("{:x}", digest) == self.md5_of_body
    }
}

impl From<aws_sdk_sqs::model::Message> for Message {
    fn from(item: aws_sdk_sqs::model::Message) -> Self {
        Message {
            body: item.body.unwrap(),
            receipt_handle: item.receipt_handle.unwrap(),
            md5_of_body: item.md5_of_body.unwrap(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_hash() {
        let message = Message {
            body: "hoge".to_string(),
            receipt_handle: "dummy".to_string(),
            md5_of_body: "ea703e7aa1efda0064eaa507d9e8ab7e".to_string(), //  md5 -s 'hoge'
        };

        assert_eq!(true, message.check_hash());
    }

    #[test]
    fn test_invalid_hash() {
        let message = Message {
            body: "hoge".to_string(),
            receipt_handle: "dummy".to_string(),
            md5_of_body: "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA".to_string(),
        };

        assert_eq!(false, message.check_hash());
    }
}
