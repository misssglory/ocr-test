use std::process::Stdio;

use anyhow::{Context, Result, bail};
use tokio::{io::AsyncWriteExt, process::Command, time::timeout};

use crate::config::OcrConfig;

pub async fn extract_text(config: &OcrConfig, image: &[u8]) -> Result<String> {
    let mut child = Command::new(&config.command)
        .arg("stdin")
        .arg("stdout")
        .arg("-l")
        .arg(&config.languages)
        .arg("--oem")
        .arg(config.oem.to_string())
        .arg("--psm")
        .arg(config.psm.to_string())
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .kill_on_drop(true)
        .spawn()
        .with_context(|| format!("failed to start OCR command `{}`", config.command))?;

    let mut stdin = child.stdin.take().context("failed to open OCR stdin")?;
    stdin
        .write_all(image)
        .await
        .context("failed to stream image to OCR process")?;
    drop(stdin);

    let output = timeout(config.timeout(), child.wait_with_output())
        .await
        .context("OCR process timed out")?
        .context("failed while waiting for OCR process")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_owned();
        bail!("OCR process exited with {}: {stderr}", output.status);
    }

    Ok(String::from_utf8_lossy(&output.stdout).trim().to_owned())
}
