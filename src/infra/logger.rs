use anyhow::Result;
use tracing;
use tracing_subscriber;
use tracing_subscriber::EnvFilter;

pub fn setup_logger() -> Result<()> {
    let filter = EnvFilter::try_from_default_env()?;
    let subscriber = tracing_subscriber::fmt().with_env_filter(filter).finish();
    tracing::subscriber::set_global_default(subscriber)?;
    Ok(())
}
