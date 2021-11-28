use crate::Config;
use anyhow::Result;
use async_trait::async_trait;
#[cfg(test)]
use mockall::automock;
use std::time::Duration;

#[cfg_attr(test, automock)]
#[async_trait]
pub trait Webhook {
    async fn post(&self, path: &str, data: String) -> Result<(bool, String)>;
}

pub struct WebhookImpl {
    pub config: Config,
}

impl WebhookImpl {
    pub fn new(config: Config) -> Self {
        WebhookImpl { config }
    }
}

#[async_trait]
impl Webhook for WebhookImpl {
    async fn post(&self, path: &str, data: String) -> Result<(bool, String)> {
        let client = reqwest::Client::new();
        let url = self.config.webhook_url.clone().join(path)?;
        let res = client
            .post(url)
            .timeout(Duration::from_secs(self.config.connection_timeout.0))
            .body(data)
            .send()
            .await?;
        Ok((res.status().is_success(), res.text().await?))
    }
}
