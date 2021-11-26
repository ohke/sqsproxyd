use anyhow::Result;
use async_trait::async_trait;
#[cfg(test)]
use mockall::automock;
use url::Url;

#[cfg_attr(test, automock)]
#[async_trait]
pub trait Webhook {
    async fn post(&self, path: &str, data: String) -> Result<String>;
}

pub struct WebhookImpl {
    pub url: Url,
}

impl WebhookImpl {
    pub fn new(url: Url) -> Self {
        WebhookImpl { url }
    }
}

#[async_trait]
impl Webhook for WebhookImpl {
    async fn post(&self, path: &str, data: String) -> Result<String> {
        let client = reqwest::Client::new();
        let url = self.url.clone().join(path)?;
        let res = client.post(url).body(data).send().await?;
        Ok(res.text().await?)
    }
}
