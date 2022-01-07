use crate::Config;
use aws_config::Config as AwsConfig;
use aws_sdk_sqs::Region;

pub async fn load_aws_config(config: &Config) -> AwsConfig {
    let loader = aws_config::from_env();
    let loader = match &config.region {
        None => loader,
        Some(region) => loader.region(Region::new(region.clone())),
    };
    loader.load().await
}
