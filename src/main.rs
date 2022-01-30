mod app;
mod domain;
mod infra;

use anyhow::{Error, Result};
use tokio::{
    signal::unix::{signal, SignalKind},
    sync::{broadcast, mpsc},
};
use tracing::info;

use crate::infra::logging::panic;
use app::daemon::Daemon;
use domain::config::Config;
use infra::{api::WebhookImpl, logging::setup_logger, sqs::AwsSqs};

#[tokio::main]
async fn main() -> Result<()> {
    // get configuration parameters
    let config = Config::new();

    setup_logger(&config.rust_log)?;

    if let Err(e) = config.validate() {
        panic("Failed to parse configuration.", e);
    }

    // run daemon
    let (shutdown_tx, shutdown_rx) = broadcast::channel(1);
    let (heartbeat_tx, mut heartbeat_rx) = mpsc::channel(1);

    let config = config.clone();

    let daemon = Daemon::new(config).await;
    tokio::spawn(async move { daemon.run(shutdown_rx, heartbeat_tx).await });

    // graceful shutdown
    if let Err(e) = receive_shutdown_signal().await {
        panic("Failed to receive shutdown signal.", e);
    }
    info!("Start to shutdown.");

    if let Err(e) = shutdown_tx.send(()) {
        panic("Failed to send shutdown message.", Error::new(e));
    };
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
