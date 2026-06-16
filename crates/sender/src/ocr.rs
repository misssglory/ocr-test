use std::{process::Stdio, sync::Arc};

use anyhow::{Context, Result, bail};
use tokio::{
    io::AsyncWriteExt,
    process::Command,
    time::timeout,
};

use crate::config::OcrConfig;

pub async fn extract_text(config: &OcrConfig, image: Arc<Vec<u8>>) -> Result<String> {
    let mut child = Command::new(&config.command)
        .arg("stdin")
        .arg("stdout")
        .arg("-l")
        .arg(&config.languages)
        .arg("--psm")
        .arg(config.psm.to_string())
        .arg("--oem")
        .arg(config.oem.to_string())
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .kill_on_drop(true)
        .spawn()
        .with_context(|| format!("failed to start {}", config.command))?;

    let mut stdin = child.stdin.take().context("failed to open tesseract stdin")?;
    let writer = tokio::spawn(async move {
        stdin.write_all(image.as_slice()).await?;
        stdin.shutdown().await?;
        Ok::<(), std::io::Error>(())
    });

    let output = timeout(config.timeout(), child.wait_with_output())
        .await
        .context("local Tesseract timed out")?
        .context("failed while waiting for local Tesseract")?;

    writer
        .await
        .context("Tesseract stdin writer task failed")?
        .context("failed to write screenshot to Tesseract stdin")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_owned();
        bail!("Tesseract exited with {}: {}", output.status, stderr);
    }

    String::from_utf8(output.stdout)
        .context("Tesseract returned non-UTF-8 output")
        .map(|text| text.trim().to_owned())
}
