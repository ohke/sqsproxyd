mod app;
mod domain;
mod infra;

use anyhow::Result;
use std::path::PathBuf;
use structopt::StructOpt;

use app::daemon::Daemon;
use domain::config::Config;
use infra::sqs::{AwsSqs, Sqs};
use infra::webhook::WebhookImpl;

#[derive(StructOpt, Debug)]
#[structopt(name = "sqsproxyd")]
pub struct Arg {
    #[structopt(short, long, parse(from_os_str))]
    env: Option<PathBuf>,
}

#[tokio::main]
async fn main() -> Result<()> {
    // get configuration parameters
    let arg = Arg::from_args();
    if let Some(v) = arg.env {
        dotenv::from_filename(v).expect("Not found env file.");
    } else {
        dotenv::dotenv().ok();
    }

    let config = Config::new()?;

    // create sqs client
    let sqs = Box::new(AwsSqs::new(config.sqs_url.to_string()).await);
    let webhook = Box::new(WebhookImpl::new(config.clone()));
    let output_sqs: Option<Box<dyn Sqs>> = match &config.output_sqs_url {
        None => None,
        Some(u) => Some(Box::new(AwsSqs::new(u.to_string()).await)),
    };
    let daemon = Daemon::new(config, sqs, webhook, output_sqs);
    daemon.run().await?;

    Ok(())
}
