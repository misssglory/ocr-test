mod config;
mod ocr;
mod server;
mod storage;

use std::{path::PathBuf, sync::Arc};

use anyhow::{Context, Result};
use clap::Parser;
use tokio::net::TcpListener;
use tracing::info;
use tracing_subscriber::EnvFilter;

use crate::{
    config::ReceiverConfig,
    server::{AppState, router},
};

#[derive(Debug, Parser)]
#[command(name = "screen-ocr-receiver")]
#[command(about = "Receive screenshots and run Tesseract OCR")]
struct Cli {
    #[arg(long, default_value = "config.receiver.toml")]
    config: PathBuf,
}

#[tokio::main]
async fn main() -> Result<()> {
    init_tracing();
    let cli = Cli::parse();
    let config = ReceiverConfig::load(&cli.config)?;
    let listener = TcpListener::bind(&config.server.bind)
        .await
        .with_context(|| format!("failed to bind receiver to {}", config.server.bind))?;

    info!(bind = %config.server.bind, "OCR receiver is listening");

    let state = Arc::new(AppState { config });
    axum::serve(listener, router(state))
        .with_graceful_shutdown(shutdown_signal())
        .await
        .context("receiver server failed")?;

    Ok(())
}

async fn shutdown_signal() {
    let _ = tokio::signal::ctrl_c().await;
    info!("stopping receiver");
}

fn init_tracing() {
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    tracing_subscriber::fmt().with_env_filter(filter).init();
}
