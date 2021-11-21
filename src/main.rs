mod app;
mod domain;
mod infra;

use anyhow::Result;
use std::path::PathBuf;
use structopt::StructOpt;

use app::daemon::Daemon;
use domain::config::Config;
use infra::sqs::AwsSqs;
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
    let sqs = AwsSqs::new(config.sqs_url.to_string()).await;
    let webhook = WebhookImpl::new(config.webhook_url.clone());
    let daemon = Daemon::new(config, Box::new(sqs), Box::new(webhook));
    daemon.run().await?;

    Ok(())
}
