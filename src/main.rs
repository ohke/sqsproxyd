mod app;
mod domain;
mod infra;

use anyhow::Result;
use std::path::PathBuf;
use structopt::StructOpt;

use app::daemon::Daemon;
use domain::config::Config;
use infra::{
    logger::setup_logger,
    sqs::{AwsSqs, Sqs},
    webhook::WebhookImpl,
};

#[derive(StructOpt, Debug)]
#[structopt(name = "sqsproxyd")]
pub struct Arg {
    #[structopt(short, long, parse(from_os_str))]
    env: Option<PathBuf>,
}

#[tokio::main]
async fn main() -> Result<()> {
    setup_logger()?;

    // get configuration parameters
    let arg = Arg::from_args();
    if let Some(v) = arg.env {
        dotenv::from_filename(v).expect("Not found env file.");
    } else {
        dotenv::dotenv().ok();
    }

    let config = Config::new()?;

    // create sqs client
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
    let _ = tokio::spawn(async move { daemon.run().await }).await?;

    Ok(())
}
