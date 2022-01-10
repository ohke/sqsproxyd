use crate::domain::arg::Arg;
use anyhow::Result;
use envy;
use serde::Deserialize;
use url::Url;

#[derive(Clone, Deserialize, Debug, PartialEq)]
pub struct Config {
    #[serde(skip)]
    pub aws_access_key_id: Option<String>,
    #[serde(skip)]
    pub aws_secret_access_key: Option<String>,
    #[serde(skip)]
    pub aws_session_token: Option<String>,
    pub aws_region: Option<String>,
    pub aws_endpoint: Option<String>,
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
        if let Some(v) = arg.aws_region {
            c.aws_region = Some(v);
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

#[cfg(test)]
mod test {
    use super::*;
    use std::env;
    use std::str::FromStr;

    fn set_env_vars() {
        env::set_var("SQSPROXYD_AWS_REGION", "us-west-1");
        env::set_var("SQSPROXYD_AWS_ENDPOINT", "http://aws-endpoint.env:2222/");
        env::set_var(
            "SQSPROXYD_SQS_URL",
            "https://sqs.us-west-1.amazonaws.com/999999999999/env-sqs-url",
        );
        env::set_var("SQSPROXYD_WEBHOOK_URL", "http://webhook-url.env:5000/");
        env::set_var(
            "SQSPROXYD_OUTPUT_SQS_URL",
            "https://sqs.us-west-1.amazonaws.com/999999999999/env-output-sqs-url",
        );
        env::set_var("SQSPROXYD_WORKER_CONCURRENCY", "2");
        env::set_var("SQSPROXYD_CONNECTION_TIMEOUT", "2");
        env::set_var("SQSPROXYD_MAX_NUMBER_OF_MESSAGES", "2");
        env::set_var("SQSPROXYD_SLEEP_SECONDS", "2");
        env::set_var(
            "SQSPROXYD_WEBHOOK_HEALTH_CHECK_URL",
            "http://webhook-health-check-url.env:5000/",
        );
        env::set_var("SQSPROXYD_WEBHOOK_HEALTH_CHECK_INTERVAL_SECONDS", "2");
        env::set_var("SQSPROXYD_CONTENT_TYPE", "application/json");
    }

    #[test]
    fn config_default_is_env() {
        set_env_vars();

        let config = Config::new(Arg {
            aws_access_key_id: None,
            aws_secret_access_key: None,
            aws_session_token: None,
            aws_region: None,
            aws_endpoint: None,
            sqs_url: None,
            webhook_url: None,
            output_sqs_url: None,
            worker_concurrency: None,
            connection_timeout: None,
            max_number_of_messages: None,
            sleep_seconds: None,
            webhook_health_check_url: None,
            webhook_health_check_interval_seconds: None,
            content_type: None,
        })
        .unwrap();

        assert_eq!(
            config,
            Config {
                aws_access_key_id: None,
                aws_secret_access_key: None,
                aws_session_token: None,
                aws_region: Some("us-west-1".to_string()),
                aws_endpoint: Some("http://aws-endpoint.env:2222/".to_string()),
                sqs_url: Url::from_str(
                    "https://sqs.us-west-1.amazonaws.com/999999999999/env-sqs-url"
                )
                .unwrap(),
                webhook_url: Url::from_str("http://webhook-url.env:5000/").unwrap(),
                output_sqs_url: Some(
                    Url::from_str(
                        "https://sqs.us-west-1.amazonaws.com/999999999999/env-output-sqs-url"
                    )
                    .unwrap()
                ),
                worker_concurrency: 2,
                connection_timeout: 2,
                max_number_of_messages: 2,
                sleep_seconds: 2,
                webhook_health_check_url: Some(
                    Url::from_str("http://webhook-health-check-url.env:5000/").unwrap()
                ),
                webhook_health_check_interval_seconds: 2,
                content_type: "application/json".to_string(),
            }
        )
    }

    #[test]
    fn config_overwritten_by_arg() {
        set_env_vars();

        let config = Config::new(Arg {
            aws_access_key_id: Some("ARGAWSACCESSKEYID".to_string()),
            aws_secret_access_key: Some("ARGAWSSECRETACCESSKEY".to_string()),
            aws_session_token: Some("ARGAWSSESSIONTOKEN".to_string()),
            aws_region: Some("us-west-2".to_string()),
            aws_endpoint: Some("http://aws-endpoint.arg:2222/".to_string()),
            sqs_url: Some(
                Url::from_str("https://sqs.us-west-2.amazonaws.com/999999999999/arg-sqs-url")
                    .unwrap(),
            ),
            webhook_url: Some(Url::from_str("http://webhook-url.arg:5000/").unwrap()),
            output_sqs_url: Some(
                Url::from_str(
                    "https://sqs.us-west-2.amazonaws.com/999999999999/arg-output-sqs-url",
                )
                .unwrap(),
            ),
            worker_concurrency: Some(3),
            connection_timeout: Some(3),
            max_number_of_messages: Some(3),
            sleep_seconds: Some(3),
            webhook_health_check_url: Some(
                Url::from_str("http://webhook-health-check-url.arg:5000/").unwrap(),
            ),
            webhook_health_check_interval_seconds: Some(3),
            content_type: Some("text/plain".to_string()),
        })
        .unwrap();

        assert_eq!(
            config,
            Config {
                aws_access_key_id: Some("ARGAWSACCESSKEYID".to_string()),
                aws_secret_access_key: Some("ARGAWSSECRETACCESSKEY".to_string()),
                aws_session_token: Some("ARGAWSSESSIONTOKEN".to_string()),
                aws_region: Some("us-west-2".to_string()),
                aws_endpoint: Some("http://aws-endpoint.arg:2222/".to_string()),
                sqs_url: Url::from_str(
                    "https://sqs.us-west-2.amazonaws.com/999999999999/arg-sqs-url"
                )
                .unwrap(),
                webhook_url: Url::from_str("http://webhook-url.arg:5000/").unwrap(),
                output_sqs_url: Some(
                    Url::from_str(
                        "https://sqs.us-west-2.amazonaws.com/999999999999/arg-output-sqs-url"
                    )
                    .unwrap()
                ),
                worker_concurrency: 3,
                connection_timeout: 3,
                max_number_of_messages: 3,
                sleep_seconds: 3,
                webhook_health_check_url: Some(
                    Url::from_str("http://webhook-health-check-url.arg:5000/").unwrap()
                ),
                webhook_health_check_interval_seconds: 3,
                content_type: "text/plain".to_string(),
            }
        )
    }
}
