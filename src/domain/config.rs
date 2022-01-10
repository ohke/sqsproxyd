use crate::domain::arg::Arg;
use anyhow::Result;
use envy;
use serde::Deserialize;
use std::env;
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
    pub webhook_health_check_url: Option<Url>,
    #[serde(default = "default_webhook_health_check_interval_seconds")]
    pub webhook_health_check_interval_seconds: u64,
    #[serde(default = "default_content_type")]
    pub content_type: String,
    #[serde(skip)]
    pub region: Option<String>,
    #[serde(skip)]
    pub aws_access_key_id: Option<String>,
    #[serde(skip)]
    pub aws_secret_access_key: Option<String>,
    #[serde(skip)]
    pub aws_session_token: Option<String>,
    #[serde(skip)]
    pub aws_endpoint: Option<String>,
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

fn default_sleep_seconds() -> u64 {
    1
}

fn default_webhook_health_check_interval_seconds() -> u64 {
    1
}

fn default_content_type() -> String {
    "application/json".to_string()
}

impl Config {
    pub fn new(arg: Arg) -> Result<Self> {
        let mut c = envy::prefixed("SQSPROXYD_").from_env::<Config>()?;

        c.aws_endpoint = match env::var("AWS_ENDPOINT") {
            Ok(v) => Some(v),
            Err(_) => None,
        };

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
        if let Some(v) = arg.webhook_health_check_url {
            c.webhook_health_check_url = Some(v);
        }
        if let Some(v) = arg.webhook_health_check_interval_seconds {
            c.webhook_health_check_interval_seconds = v;
        }
        if let Some(v) = arg.content_type {
            c.content_type = v;
        }
        if let Some(v) = arg.region {
            c.region = Some(v);
        }
        if let Some(v) = arg.aws_access_key_id {
            c.aws_access_key_id = Some(v);
        }
        if let Some(v) = arg.aws_secret_access_key {
            c.aws_secret_access_key = Some(v);
        }
        if let Some(v) = arg.aws_session_token {
            c.aws_session_token = Some(v);
        }
        if let Some(v) = arg.aws_endpoint {
            c.aws_endpoint = Some(v);
        }

        Ok(c)
    }
}
