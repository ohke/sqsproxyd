use crate::{ApiImpl, AwsSqs};
use anyhow::{anyhow, Result};
use std::borrow::Borrow;
use tokio::{
    sync::{broadcast, mpsc},
    time::{sleep, Duration},
};
use tracing::{debug, error, warn};
use url::Url;

use crate::domain::config::Config;
use crate::domain::message::Message;
use crate::infra::api::Api;
use crate::infra::logging::panic;
use crate::infra::sqs::Sqs;

pub struct Daemon {
    config: Config,
    sqs: Box<dyn Sqs + Send + Sync>,
    api: Box<dyn Api + Send + Sync>,
}

impl Daemon {
    pub async fn new(config: Config) -> Self {
        Daemon {
            config: config.clone(),
            sqs: Box::new(AwsSqs::new(config.sqs_url.to_string(), &config).await),
            api: Box::new(ApiImpl::new(config.clone())),
        }
    }

    pub async fn run(
        self,
        mut shutdown_rx: broadcast::Receiver<()>,
        _heartbeat_tx: mpsc::Sender<()>,
    ) -> Result<()> {
        // wait for health check
        if let Some(url) = &self.config.api_health_url {
            tokio::select! {
                result = Self::healthcheck(self.api.borrow(), url, self.config.api_health_interval_seconds) => {
                    match result {
                        Ok(v) => v,
                        Err(e) => panic("Failed to pass health check of the API.", e),
                    }
                },
                _ = shutdown_rx.recv() => return Ok(()),
            }
        }

        // create workers
        let (tx, rx) = async_channel::bounded::<Message>(self.config.num_workers);
        let (worker_waiting_tx, mut worker_waiting_rx) =
            mpsc::channel::<()>(self.config.num_workers);
        let (worker_shutdown_tx, _) = broadcast::channel(1);
        let (worker_heartbeat_tx, mut worker_heartbeat_rx) = mpsc::channel::<()>(1);

        for _ in 0..self.config.num_workers {
            let config = self.config.clone();
            let sqs = Box::new(AwsSqs::new(config.sqs_url.to_string(), &config).await);
            let api = Box::new(ApiImpl::new(self.config.clone()));
            let output_sqs: Option<Box<dyn Sqs + Send + Sync>> = match &self.config.output_sqs_url {
                None => None,
                Some(u) => Some(Box::new(AwsSqs::new(u.to_string(), &config).await)),
            };
            let rx = rx.clone();
            let waiting_tx = worker_waiting_tx.clone();
            let shutdown_rx = worker_shutdown_tx.subscribe();
            let heartbeat_tx = worker_heartbeat_tx.clone();

            tokio::spawn(async move {
                Self::poll_process(
                    sqs,
                    api,
                    output_sqs,
                    rx,
                    waiting_tx,
                    shutdown_rx,
                    heartbeat_tx,
                )
                .await
            });
            let _ = worker_waiting_tx.send(()).await;
        }

        drop(worker_heartbeat_tx);

        // receive SQS message
        loop {
            let _ = worker_waiting_rx.recv().await;
            tokio::select! {
                result = self.sqs.receive_messages() => {
                    match result {
                        Ok(response) => {
                            match response {
                                Some(messages) => {
                                    if messages.is_empty() {
                                        warn!("Empty message received. Sleep.");
                                        Self::sleep(self.config.sleep_msec).await;
                                        if let Err(e) = worker_waiting_tx.send(()).await {
                                           error!("Failed to send waiting queue. ({:?})", e);
                                        }
                                    } else {
                                        for message in messages {
                                            debug!("Received message: {:?}", message);

                                            let r = tx.send(message).await;
                                            if r.is_err() {
                                                error!("Failed to send received message to worker.");
                                            }
                                            r?;
                                        }
                                    }
                                }
                                None => {
                                    debug!("No received message. Sleep.");
                                    Self::sleep(self.config.sleep_msec).await;
                                    if let Err(e) = worker_waiting_tx.send(()).await {
                                        error!("Failed to send waiting queue. ({:?})", e);
                                    }
                                }
                            }
                        },
                        Err(e) => {
                            error!("Failed to receive messages from SQS. ({:?})", e);
                            Self::sleep(self.config.sleep_msec).await;
                            if let Err(e) = worker_waiting_tx.send(()).await {
                                error!("Failed to send waiting queue. ({:?})", e);
                            }
                        }
                    }
                }
                _ = shutdown_rx.recv() => {
                    let r = worker_shutdown_tx.send(());
                    if r.is_ok() {
                        let _ = worker_heartbeat_rx.recv().await;
                    } else {
                        error!("Failed to send shutdown message to worker.");
                    }
                    return Ok(());
                }
            }
        }
    }

    async fn healthcheck(api: &'_ (dyn Api + Send + Sync), url: &Url, seconds: u64) -> Result<()> {
        loop {
            if api.get(url).await.is_ok() {
                break;
            }

            Self::sleep(seconds).await;
        }

        Ok(())
    }

    async fn poll_process(
        sqs: Box<dyn Sqs + Send + Sync>,
        api: Box<dyn Api + Send + Sync>,
        output_sqs: Option<Box<dyn Sqs + Send + Sync>>,
        rx: async_channel::Receiver<Message>,
        waiting_tx: mpsc::Sender<()>,
        mut shutdown_rx: broadcast::Receiver<()>,
        _heartbeat_tx: mpsc::Sender<()>,
    ) -> Result<()> {
        loop {
            tokio::select! {
                result = rx.recv() => {
                    match result {
                        Ok(message) => {
                            debug!("Processing message: {:?}", message);

                            if message.check_hash() {
                                match Self::process_message(message.clone(), sqs.borrow(), api.borrow(), &output_sqs).await {
                                    Ok(()) => debug!("Succeeded to process message. ({})", message.message_id),
                                    Err(e) => error!("Failed to process message. ({}, {:?})", message.message_id, e),
                                };
                            } else {
                                warn!("Mismatch message MD5 digest. ({})", message.message_id);
                            }
                        }
                        Err(e) => {
                            error!("Failed to receive message. ({:?})", e);
                        }
                    }

                    if let Err(e) = waiting_tx.send(()).await {
                        error!("Failed to send waiting queue. ({:?})", e);
                    }
                }
                _ = shutdown_rx.recv() => return Ok(()),
            }
        }
    }

    async fn process_message(
        message: Message,
        sqs: &'_ (dyn Sqs + Send + Sync),
        api: &'_ (dyn Api + Send + Sync),
        output_sqs: &Option<Box<dyn Sqs + Send + Sync>>,
    ) -> Result<()> {
        let (is_succeeded, res) = api.post(message.body.clone(), &message.message_id).await?;
        if !is_succeeded {
            return Err(anyhow!("API returns failed status response."));
        }

        if output_sqs.is_some() {
            output_sqs.as_ref().unwrap().send_message(res).await?;
        }

        sqs.delete_message(message.receipt_handle).await?;

        Ok(())
    }

    async fn sleep(milliseconds: u64) {
        sleep(Duration::from_millis(milliseconds)).await;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::message::*;
    use crate::infra::api::*;
    use crate::infra::sqs::*;
    use anyhow::anyhow;
    use mockall::predicate::*;
    use std::borrow::Borrow;
    use std::str::FromStr;

    #[tokio::test]
    async fn test_process_message_with_output() {
        dotenv::from_filename("env/test.env").expect("Not found env file.");

        let mut sqs = MockSqs::new();
        sqs.expect_send_message().times(0).returning(|_| Ok(()));
        sqs.expect_delete_message()
            .with(eq("receipt_handle".to_string()))
            .times(1)
            .returning(|_| Ok(()));
        let sqs: Box<dyn Sqs + Send + Sync> = Box::new(sqs);

        let mut api = MockApi::new();
        api.expect_post().times(1).returning(|_, message_id| {
            assert_eq!(message_id, "message_id");
            Ok((true, "result".to_string()))
        });
        let api: Box<dyn Api + Send + Sync> = Box::new(api);

        let mut output_sqs = MockSqs::new();
        output_sqs.expect_receive_messages().times(0);
        output_sqs
            .expect_send_message()
            .with(eq("result".to_string()))
            .times(1)
            .returning(|_| Ok(()));
        output_sqs.expect_delete_message().times(0);
        let output_sqs: Option<Box<dyn Sqs + Send + Sync>> = Some(Box::new(output_sqs));

        let message = Message {
            receipt_handle: "receipt_handle".to_string(),
            body: "{\"key1\": 1}".to_string(),
            md5_of_body: "dummy".to_string(),
            message_id: "message_id".to_string(),
        };

        Daemon::process_message(message, sqs.borrow(), api.borrow(), &output_sqs)
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn test_process_message_without_output() {
        dotenv::from_filename("env/test.env").expect("Not found env file.");

        let mut sqs = MockSqs::new();
        sqs.expect_send_message().times(0).returning(|_| Ok(()));
        sqs.expect_delete_message()
            .with(eq("receipt_handle".to_string()))
            .times(1)
            .returning(|_| Ok(()));
        let sqs: Box<dyn Sqs + Send + Sync> = Box::new(sqs);

        let mut api = MockApi::new();
        api.expect_post().times(1).returning(|_, message_id| {
            assert_eq!(message_id, "message_id");
            Ok((true, "result".to_string()))
        });
        let api: Box<dyn Api + Send + Sync> = Box::new(api);

        let output_sqs = None;

        let message = Message {
            receipt_handle: "receipt_handle".to_string(),
            body: "{\"key1\": 1}".to_string(),
            md5_of_body: "dummy".to_string(),
            message_id: "message_id".to_string(),
        };

        Daemon::process_message(message, sqs.borrow(), api.borrow(), &output_sqs)
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn test_process_message_if_failed_not_deleted() {
        let mut sqs = MockSqs::new();
        sqs.expect_delete_message().times(0);
        let sqs: Box<dyn Sqs + Send + Sync> = Box::new(sqs);

        let mut api = MockApi::new();
        api.expect_post().times(1).returning(|_, message_id| {
            assert_eq!(message_id, "message_id");
            Ok((false, "result".to_string()))
        });
        let api: Box<dyn Api + Send + Sync> = Box::new(api);

        let mut output_sqs = MockSqs::new();
        output_sqs.expect_receive_messages().times(0);
        output_sqs.expect_send_message().times(0);
        output_sqs.expect_delete_message().times(0);
        let output_sqs: Option<Box<dyn Sqs + Send + Sync>> = Some(Box::new(output_sqs));

        let message = Message {
            receipt_handle: "receipt_handle".to_string(),
            body: "{\"key1\": 1}".to_string(),
            md5_of_body: "dummy".to_string(),
            message_id: "message_id".to_string(),
        };

        assert!(
            Daemon::process_message(message, sqs.borrow(), api.borrow(), &output_sqs)
                .await
                .is_err()
        );
    }

    #[tokio::test]
    async fn test_healthcheck() {
        let mut api = MockApi::new();
        api.expect_get()
            .times(3)
            .returning(|_| Err(anyhow!("Error")))
            .times(1)
            .returning(|_| Ok(()));
        let api: Box<dyn Api + Send + Sync> = Box::new(api);

        Daemon::healthcheck(
            api.borrow(),
            &Url::from_str("http://dummy:1234/").unwrap(),
            1,
        )
        .await
        .unwrap();
    }
}
