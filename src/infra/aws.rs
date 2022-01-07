use aws_config::Config as AwsConfig;

pub async fn load_aws_config() -> AwsConfig {
    let loader = aws_config::from_env();
    loader.load().await
}
