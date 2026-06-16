use anyhow::{Context, Result, bail};
use reqwest::Client;
use screen_ocr_common::{ErrorResponse, TextReceipt, TextSubmission};

use crate::config::SenderConfig;

#[derive(Clone)]
pub struct TextClient {
    http: Client,
    config: SenderConfig,
}

impl TextClient {
    pub fn new(config: SenderConfig) -> Result<Self> {
        let http = Client::builder()
            .timeout(config.server.timeout())
            .build()
            .context("failed to build HTTP client")?;
        Ok(Self { http, config })
    }

    pub async fn send(&self, submission: &TextSubmission) -> Result<TextReceipt> {
        let response = self
            .http
            .post(self.config.server.text_url())
            .bearer_auth(&self.config.server.token)
            .json(submission)
            .send()
            .await
            .context("failed to send OCR text to receiver")?;

        let status = response.status();
        let body = response.bytes().await.context("failed to read receiver response")?;

        if !status.is_success() {
            let message = serde_json::from_slice::<ErrorResponse>(&body)
                .map(|value| value.error)
                .unwrap_or_else(|_| String::from_utf8_lossy(&body).into_owned());
            bail!("receiver returned {status}: {message}");
        }

        serde_json::from_slice(&body).context("receiver returned invalid JSON")
    }
}
