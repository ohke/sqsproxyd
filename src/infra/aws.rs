use crate::Config;
use aws_config::Config as AwsConfig;
use aws_sdk_sqs::Region;
use aws_types::Credentials;

pub async fn load_aws_config(config: &Config) -> AwsConfig {
    let loader = aws_config::from_env();
    let loader = match &config.region {
        None => loader,
        Some(region) => loader.region(Region::new(region.clone())),
    };
    let loader = match &config.aws_access_key_id {
        None => loader,
        Some(aws_access_key_id) => loader.credentials_provider(Credentials::from_keys(
            aws_access_key_id.clone(),
            config.aws_secret_access_key.as_ref().unwrap(),
            config.aws_session_token.clone(),
        )),
    };
    loader.load().await
}
