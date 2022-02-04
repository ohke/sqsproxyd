use anyhow::{anyhow, Result};
use http::Uri;
use structopt::StructOpt;
use url::Url;

#[derive(Clone, Debug, PartialEq, StructOpt)]
#[structopt(name = "sqsproxyd")]
pub struct Config {
    #[structopt(long, env = "AWS_ACCESS_KEY_ID")]
    pub aws_access_key_id: Option<String>,
    #[structopt(long, env = "AWS_SECRET_ACCESS_KEY")]
    pub aws_secret_access_key: Option<String>,
    #[structopt(long, env = "AWS_SESSION_TOKEN")]
    pub aws_session_token: Option<String>,
    #[structopt(long, env = "SQSPROXYD_AWS_REGION")]
    pub aws_region: Option<String>,
    #[structopt(long, env = "SQSPROXYD_AWS_ENDPOINT")]
    pub aws_endpoint: Option<Uri>,
    #[structopt(long, env = "SQSPROXYD_SQS_URL")]
    pub sqs_url: Url,
    #[structopt(long, env = "SQSPROXYD_API_URL")]
    pub api_url: Url,
    #[structopt(long, env = "SQSPROXYD_OUTPUT_SQS_URL")]
    pub output_sqs_url: Option<Url>,
    #[structopt(long, env = "SQSPROXYD_NUM_WORKERS", default_value = "1")]
    pub num_workers: usize,
    #[structopt(long, env = "SQSPROXYD_API_TIMEOUT_MSEC", default_value = "30000")]
    pub api_timeout_msec: u64,
    #[structopt(long, env = "SQSPROXYD_SLEEP_MSEC", default_value = "1000")]
    pub sleep_msec: u64,
    #[structopt(long, env = "SQSPROXYD_API_HEALTH_URL")]
    pub api_health_url: Option<Url>,
    #[structopt(
        long,
        env = "SQSPROXYD_API_HEALTH_INTERVAL_SECONDS",
        default_value = "1"
    )]
    pub api_health_interval_seconds: u64,
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
        if self.aws_endpoint.is_some()
            && (self.aws_access_key_id.is_none() || self.aws_secret_access_key.is_none())
        {
            return Err(anyhow!("If `--aws-endpoint` is set, `--aws-access-key-id` and `--aws-secret-access-key` should be set."));
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
        env::set_var("AWS_ACCESS_KEY_ID", "AWSACCESSKEY");
        env::set_var("AWS_SECRET_ACCESS_KEY", "AWSSECRETACCESSKEY");
        env::set_var("AWS_SESSION_TOKEN", "AWSSESSIONTOKEN");
        env::set_var("SQSPROXYD_AWS_REGION", "us-west-1");
        env::set_var("SQSPROXYD_AWS_ENDPOINT", "http://aws-endpoint.env:2222/");
        env::set_var(
            "SQSPROXYD_SQS_URL",
            "https://sqs.us-west-1.amazonaws.com/999999999999/env-sqs-url",
        );
        env::set_var("SQSPROXYD_API_URL", "http://api-url.env:5000/");
        env::set_var(
            "SQSPROXYD_OUTPUT_SQS_URL",
            "https://sqs.us-west-1.amazonaws.com/999999999999/env-output-sqs-url",
        );
        env::set_var("SQSPROXYD_NUM_WORKERS", "2");
        env::set_var("SQSPROXYD_API_TIMEOUT_MSEC", "2");
        env::set_var("SQSPROXYD_SLEEP_MSEC", "2");
        env::set_var(
            "SQSPROXYD_API_HEALTH_URL",
            "http://api-health-check-url.env:5000/",
        );
        env::set_var("SQSPROXYD_API_HEALTH_INTERVAL_SECONDS", "2");
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
                aws_access_key_id: Some("AWSACCESSKEY".to_string()),
                aws_secret_access_key: Some("AWSSECRETACCESSKEY".to_string()),
                aws_session_token: Some("AWSSESSIONTOKEN".to_string()),
                aws_region: Some("us-west-1".to_string()),
                aws_endpoint: Some(Uri::from_str("http://aws-endpoint.env:2222/").unwrap()),
                sqs_url: Url::from_str(
                    "https://sqs.us-west-1.amazonaws.com/999999999999/env-sqs-url"
                )
                .unwrap(),
                api_url: Url::from_str("http://api-url.env:5000/").unwrap(),
                output_sqs_url: Some(
                    Url::from_str(
                        "https://sqs.us-west-1.amazonaws.com/999999999999/env-output-sqs-url"
                    )
                    .unwrap()
                ),
                num_workers: 2,
                api_timeout_msec: 2,
                sleep_msec: 2,
                api_health_url: Some(
                    Url::from_str("http://api-health-check-url.env:5000/").unwrap()
                ),
                api_health_interval_seconds: 2,
                content_type: "application/json".to_string(),
                rust_log: "INFO".to_string(),
            }
        )
    }
}
