use anyhow::Result;
use std::str::FromStr;
use tracing;
use tracing_subscriber;
use tracing_subscriber::EnvFilter;

pub fn setup_logger(rust_log: &str) -> Result<()> {
    let filter = EnvFilter::from_str(rust_log)?;
    let subscriber = tracing_subscriber::fmt().with_env_filter(filter).finish();
    tracing::subscriber::set_global_default(subscriber)?;
    Ok(())
}
