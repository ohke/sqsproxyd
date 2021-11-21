use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
pub struct Message {
    path: String,
    data: String,
    context: Option<String>,
}
