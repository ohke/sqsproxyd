use anyhow::Result;
use envy;
use serde::Deserialize;
use url::Url;

#[derive(Clone, Deserialize, Debug)]
pub struct Config {
    pub sqs_url: Url,
    pub webhook_url: Url,
    pub output_sqs_url: Option<Url>,
    #[serde(default = "default_worker_concurrency")]
    pub worker_concurrency: usize,
    #[serde(default)]
    pub connection_timeout: Timeout,
}

fn default_worker_concurrency() -> usize {
    1
}

#[derive(Clone, Copy, Deserialize, Debug)]
pub struct Timeout(pub u64);
impl Default for Timeout {
    fn default() -> Self {
        Timeout(30)
    }
}

impl Config {
    pub fn new() -> Result<Self> {
        let v = envy::prefixed("SQSPROXYD_").from_env::<Config>()?;
        Ok(v)
    }
}
