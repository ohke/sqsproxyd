use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
pub struct Message {
    pub path: String,
    pub data: String,
    pub context: Option<String>,
}
