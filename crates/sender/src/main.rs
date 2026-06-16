mod capture;
mod client;
mod config;

use std::path::PathBuf;

use anyhow::Result;
use clap::{Parser, Subcommand};
use tracing::{info, warn};
use tracing_subscriber::EnvFilter;

use crate::{client::OcrClient, config::SenderConfig};

#[derive(Debug, Parser)]
#[command(name = "screen-ocr-sender")]
#[command(about = "Capture a screen and send it to a remote OCR receiver")]
struct Cli {
    #[arg(long, default_value = "config.sender.toml")]
    config: PathBuf,

    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Print available monitors and their indexes.
    ListMonitors,
    /// Capture and send one screenshot.
    Once,
    /// Capture continuously and send changed frames.
    Watch,
}

#[tokio::main]
async fn main() -> Result<()> {
    init_tracing();
    let cli = Cli::parse();

    if matches!(cli.command, Command::ListMonitors) {
        return capture::list_monitors();
    }

    let config = SenderConfig::load(&cli.config)?;
    let client = OcrClient::new(config.clone())?;

    match cli.command {
        Command::ListMonitors => unreachable!(),
        Command::Once => send_once(&client, &config).await?,
        Command::Watch => watch(client, config).await?,
    }

    Ok(())
}

async fn send_once(client: &OcrClient, config: &SenderConfig) -> Result<()> {
    let frame = capture::capture(&config.capture)?;
    info!(
        width = frame.width,
        height = frame.height,
        bytes = frame.bytes.len(),
        sha256 = %frame.sha256,
        "captured screenshot"
    );

    let response = client.send(frame).await?;
    println!("{}", serde_json::to_string_pretty(&response)?);
    Ok(())
}

async fn watch(client: OcrClient, config: SenderConfig) -> Result<()> {
    let mut previous_hash: Option<String> = None;
    let mut interval = tokio::time::interval(std::time::Duration::from_millis(
        config.capture.interval_ms.max(50),
    ));

    loop {
        tokio::select! {
            _ = tokio::signal::ctrl_c() => {
                info!("stopping sender");
                return Ok(());
            }
            _ = interval.tick() => {
                match capture::capture(&config.capture) {
                    Ok(frame) => {
                        if !config.capture.send_unchanged
                            && previous_hash.as_deref() == Some(frame.sha256.as_str())
                        {
                            continue;
                        }

                        previous_hash = Some(frame.sha256.clone());
                        match client.send(frame).await {
                            Ok(response) => {
                                println!("{}", serde_json::to_string(&response)?);
                            }
                            Err(error) => warn!(%error, "failed to send frame"),
                        }
                    }
                    Err(error) => warn!(%error, "failed to capture frame"),
                }
            }
        }
    }
}

fn init_tracing() {
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    tracing_subscriber::fmt().with_env_filter(filter).init();
}
