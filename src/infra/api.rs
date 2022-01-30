use crate::Config;
use anyhow::Result;
use async_trait::async_trait;
#[cfg(test)]
use mockall::automock;
use std::time::Duration;
use url::Url;

#[cfg_attr(test, automock)]
#[async_trait]
pub trait Api {
    async fn get(&self, url: &Url) -> Result<()>;
    async fn post(&self, data: String, message_id: &str) -> Result<(bool, String)>;
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
impl Api for WebhookImpl {
    async fn get(&self, url: &Url) -> Result<()> {
        let client = reqwest::Client::new();
        let _ = client
            .get(url.clone())
            .header(
                reqwest::header::USER_AGENT,
                format!("sqsdproxy/{}", env!("CARGO_PKG_VERSION")),
            )
            .timeout(Duration::from_secs(3600))
            .send()
            .await?;
        Ok(())
    }

    async fn post(&self, data: String, message_id: &str) -> Result<(bool, String)> {
        let client = reqwest::Client::new();
        let res = client
            .post(self.config.webhook_url.clone())
            .header(
                reqwest::header::USER_AGENT,
                format!("sqsdproxy/{}", env!("CARGO_PKG_VERSION")),
            )
            .header(reqwest::header::CONTENT_TYPE, &self.config.content_type)
            .header("X-SQSPROXYD-MESSAGE-ID", message_id)
            .timeout(Duration::from_secs(self.config.connection_timeout))
            .body(data)
            .send()
            .await?;
        Ok((res.status().is_success(), res.text().await?))
    }
}
