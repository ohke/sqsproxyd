use anyhow::Result;
use envy;
use serde::Deserialize;
use url::Url;

#[derive(Deserialize, Debug)]
pub struct Config {
    sqs_url: Url,
    webhook_url: Url,
}

impl Config {
    pub fn new() -> Result<Self> {
        let v = envy::prefixed("SQSPROXYD_").from_env::<Config>()?;
        Ok(v)
    }
}
