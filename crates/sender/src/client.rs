use anyhow::{Context, Result, bail};
use reqwest::{Client, header::CONTENT_TYPE};
use screen_ocr_common::{
    DEVICE_ID_HEADER, ErrorResponse, IMAGE_HEIGHT_HEADER, IMAGE_SHA256_HEADER, IMAGE_WIDTH_HEADER,
    OcrResponse,
};

use crate::{capture::CapturedFrame, config::SenderConfig};

pub struct OcrClient {
    http: Client,
    config: SenderConfig,
}

impl OcrClient {
    pub fn new(config: SenderConfig) -> Result<Self> {
        let http = Client::builder()
            .timeout(config.server.timeout())
            .build()
            .context("failed to build HTTP client")?;

        Ok(Self { http, config })
    }

    pub async fn send(&self, frame: CapturedFrame) -> Result<OcrResponse> {
        let response = self
            .http
            .post(self.config.server.ocr_url())
            .bearer_auth(&self.config.server.token)
            .header(CONTENT_TYPE, frame.mime)
            .header(DEVICE_ID_HEADER, &self.config.capture.device_id)
            .header(IMAGE_SHA256_HEADER, &frame.sha256)
            .header(IMAGE_WIDTH_HEADER, frame.width.to_string())
            .header(IMAGE_HEIGHT_HEADER, frame.height.to_string())
            .body(frame.bytes)
            .send()
            .await
            .context("failed to send screenshot to OCR receiver")?;

        let status = response.status();
        let body = response
            .bytes()
            .await
            .context("failed to read receiver response")?;

        if !status.is_success() {
            let message = serde_json::from_slice::<ErrorResponse>(&body)
                .map(|value| value.error)
                .unwrap_or_else(|_| String::from_utf8_lossy(&body).into_owned());
            bail!("receiver returned {status}: {message}");
        }

        serde_json::from_slice(&body).context("receiver returned invalid JSON")
    }
}
