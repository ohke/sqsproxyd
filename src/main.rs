mod app;
mod domain;
mod infra;

use anyhow::Result;
use structopt::StructOpt;

use app::daemon::Daemon;
use domain::{arg::Arg, config::Config};
use infra::{
    logger::setup_logger,
    sqs::{AwsSqs, Sqs},
    webhook::WebhookImpl,
};

#[tokio::main]
async fn main() -> Result<()> {
    setup_logger()?;

    // get configuration parameters
    let arg = Arg::from_args();
    let config = Config::new(arg)?;

    let mut handles = vec![];

    for _ in 0..config.worker_concurrency {
        let config = config.clone();
        let sqs =
            Box::new(AwsSqs::new(config.sqs_url.to_string(), config.max_number_of_messages).await);
        let webhook = Box::new(WebhookImpl::new(config.clone()));
        let output_sqs: Option<Box<dyn Sqs + Send + Sync>> = match &config.output_sqs_url {
            None => None,
            Some(u) => Some(Box::new(
                AwsSqs::new(u.to_string(), config.max_number_of_messages).await,
            )),
        };

        let daemon = Daemon::new(config, sqs, webhook, output_sqs);
        handles.push(tokio::spawn(async move { daemon.run().await }));
    }

    for handle in handles {
        let _ = handle.await.unwrap();
    }

    Ok(())
}
