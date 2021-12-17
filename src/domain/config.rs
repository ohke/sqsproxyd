use crate::domain::arg::Arg;
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
    #[serde(default = "default_connection_timeout")]
    pub connection_timeout: u64,
    #[serde(default = "default_max_number_of_messages")]
    pub max_number_of_messages: usize,
    #[serde(default = "default_sleep_seconds")]
    pub sleep_seconds: u64,
    pub webhook_health_check_path: Option<String>,
}

fn default_worker_concurrency() -> usize {
    1
}

fn default_max_number_of_messages() -> usize {
    1
}

fn default_connection_timeout() -> u64 {
    30
}

impl Config {
    pub fn new(arg: Arg) -> Result<Self> {
        let mut c = envy::prefixed("SQSPROXYD_").from_env::<Config>()?;

        if let Some(v) = arg.sqs_url {
            c.sqs_url = v;
        }
        if let Some(v) = arg.webhook_url {
            c.webhook_url = v;
        }
        if let Some(v) = arg.output_sqs_url {
            c.output_sqs_url = Some(v);
        }
        if let Some(v) = arg.worker_concurrency {
            c.worker_concurrency = v;
        }
        if let Some(v) = arg.connection_timeout {
            c.connection_timeout = v;
        }
        if let Some(v) = arg.max_number_of_messages {
            c.max_number_of_messages = v;
        }
        if let Some(v) = arg.sleep_seconds {
            c.sleep_seconds = v;
        }
        if let Some(v) = arg.webhook_health_check_path {
            c.webhook_health_check_path = Some(v);
        }

        Ok(c)
    }
}

fn default_sleep_seconds() -> u64 {
    1
}
