mod app;
mod domain;
mod infra;

use anyhow::Result;
use structopt::StructOpt;
use tokio::{
    signal::unix::{signal, SignalKind},
    sync::{broadcast, mpsc},
};
use tracing::info;

use app::daemon::Daemon;
use domain::{arg::Arg, config::Config};
use infra::{logger::setup_logger, sqs::AwsSqs, webhook::WebhookImpl};

#[tokio::main]
async fn main() -> Result<()> {
    setup_logger()?;

    // get configuration parameters
    let arg = Arg::from_args();
    let config = Config::new(arg)?;

    // run daemon
    let (shutdown_tx, shutdown_rx) = broadcast::channel(1);
    let (heartbeat_tx, mut heartbeat_rx) = mpsc::channel(1);

    let config = config.clone();

    let daemon = Daemon::new(config).await;
    tokio::spawn(async move { daemon.run(shutdown_rx, heartbeat_tx).await });

    // graceful shutdown
    receive_shutdown_signal().await.unwrap();
    info!("Start to shutdown.");

    shutdown_tx.send(()).unwrap();
    let _ = heartbeat_rx.recv().await;
    info!("Terminated.");

    Ok(())
}

async fn receive_shutdown_signal() -> Result<()> {
    let mut sigint = signal(SignalKind::interrupt())?;
    let mut sigterm = signal(SignalKind::terminate())?;
    tokio::select! {
        _ = sigint.recv() => info!("Receives SIGINT signal."),
        _ = sigterm.recv() => info!("Receives SIGTERM signal.")
    }
    Ok(())
}
