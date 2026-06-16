mod capture;
mod client;
mod config;
mod server;

use std::{path::PathBuf, sync::Arc};

use anyhow::{Context, Result};
use clap::Parser;
use tokio::net::TcpListener;
use tracing::info;
use tracing_subscriber::EnvFilter;

use crate::{client::OcrClient, config::SenderConfig, server::AppState};

#[derive(Debug, Parser)]
#[command(name = "screen-ocr-sender")]
#[command(about = "Expose a local trigger that captures the full primary screen and sends it to OCR")]
struct Cli {
    #[arg(long, default_value = "config.sender.toml")]
    config: PathBuf,
}

#[tokio::main]
async fn main() -> Result<()> {
    init_tracing();

    let cli = Cli::parse();
    let config = SenderConfig::load(&cli.config)?;
    let client = OcrClient::new(config.clone())?;
    let bind = config.trigger.bind.clone();

    let state = Arc::new(AppState { config, client });
    let listener = TcpListener::bind(&bind)
        .await
        .with_context(|| format!("failed to bind sender trigger server to {bind}"))?;

    info!(%bind, "sender trigger server is listening");

    axum::serve(listener, server::router(state))
        .with_graceful_shutdown(shutdown_signal())
        .await
        .context("sender trigger server failed")?;

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
