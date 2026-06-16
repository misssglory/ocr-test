mod config;
mod server;
mod storage;

use std::{path::PathBuf, sync::Arc};

use anyhow::{Context, Result};
use clap::Parser;
use tokio::net::TcpListener;
use tracing::info;
use tracing_subscriber::EnvFilter;

use crate::{config::ReceiverConfig, server::AppState};

#[derive(Debug, Parser)]
#[command(name = "screen-ocr-receiver")]
#[command(about = "Receive OCR text produced by a remote sender")]
struct Cli {
    #[arg(long, default_value = "config.receiver.toml")]
    config: PathBuf,
}

#[tokio::main]
async fn main() -> Result<()> {
    init_tracing();

    let cli = Cli::parse();
    let config = ReceiverConfig::load(&cli.config)?;
    let bind = config.server.bind.clone();
    let listener = TcpListener::bind(&bind)
        .await
        .with_context(|| format!("failed to bind receiver to {bind}"))?;

    info!(%bind, "text receiver is listening");

    axum::serve(listener, server::router(Arc::new(AppState { config })))
        .with_graceful_shutdown(shutdown_signal())
        .await
        .context("receiver server failed")?;

    Ok(())
}

async fn shutdown_signal() {
    if let Err(error) = tokio::signal::ctrl_c().await {
        tracing::error!(%error, "failed to install Ctrl-C handler");
    }
}

fn init_tracing() {
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    tracing_subscriber::fmt().with_env_filter(filter).init();
}
