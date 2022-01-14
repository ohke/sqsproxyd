use anyhow::{anyhow, Result};
use structopt::StructOpt;
use url::Url;

#[derive(Clone, Debug, PartialEq, StructOpt)]
#[structopt(name = "sqsproxyd")]
pub struct Config {
    #[structopt(long)]
    pub aws_access_key_id: Option<String>,
    #[structopt(long)]
    pub aws_secret_access_key: Option<String>,
    #[structopt(long)]
    pub aws_session_token: Option<String>,
    #[structopt(long, env = "SQSPROXYD_AWS_REGION")]
    pub aws_region: Option<String>,
    #[structopt(long, env = "SQSPROXYD_AWS_ENDPOINT")]
    pub aws_endpoint: Option<String>,
    #[structopt(long, env = "SQSPROXYD_SQS_URL")]
    pub sqs_url: Url,
    #[structopt(long, env = "SQSPROXYD_WEBHOOK_URL")]
    pub webhook_url: Url,
    #[structopt(long, env = "SQSPROXYD_OUTPUT_SQS_URL")]
    pub output_sqs_url: Option<Url>,
    #[structopt(long, env = "SQSPROXYD_WORKER_CONCURRENCY", default_value = "1")]
    pub worker_concurrency: usize,
    #[structopt(long, env = "SQSPROXYD_CONNECTION_TIMEOUT", default_value = "30")]
    pub connection_timeout: u64,
    #[structopt(long, env = "SQSPROXYD_MAX_NUMBER_OF_MESSAGES", default_value = "1")]
    pub max_number_of_messages: usize,
    #[structopt(long, env = "SQSPROXYD_SLEEP_SECONDS", default_value = "1")]
    pub sleep_seconds: u64,
    #[structopt(long, env = "SQSPROXYD_WEBHOOK_HEALTH_CHECK_URL")]
    pub webhook_health_check_url: Option<Url>,
    #[structopt(
        long,
        env = "SQSPROXYD_WEBHOOK_HEALTH_CHECK_INTERVAL_SECONDS",
        default_value = "1"
    )]
    pub webhook_health_check_interval_seconds: u64,
    #[structopt(
        long,
        env = "SQSPROXYD_CONTENT_TYPE",
        default_value = "application/json"
    )]
    pub content_type: String,
    #[structopt(long, env = "SQSPROXYD_RUST_LOG", default_value = "WARN")]
    pub rust_log: String,
}

impl Config {
    pub fn new() -> Self {
        Self::from_args()
    }

    pub fn validate(&self) -> Result<()> {
        if !(1 <= self.max_number_of_messages && self.max_number_of_messages <= 10) {
            return Err(anyhow!(
                "`max_number_of_messages` should be >= 1 and <= 10."
            ));
        }

        Ok(())
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
        env::set_var("SQSPROXYD_RUST_LOG", "INFO")
    }

    #[test]
    fn config_default_is_env() {
        set_env_vars();

        let config = Config::new();
        config.validate().unwrap();

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
                rust_log: "INFO".to_string(),
            }
        )
    }
}
