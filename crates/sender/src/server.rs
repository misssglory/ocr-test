use std::{sync::Arc, time::Instant};

use axum::{
    Json, Router,
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::post,
};
use screen_ocr_common::{ErrorResponse, TextReceipt, TextSubmission};
use tracing::info;

use crate::{capture, client::TextClient, config::SenderConfig, ocr};

#[derive(Clone)]
pub struct AppState {
    pub config: SenderConfig,
    pub client: TextClient,
}

pub fn router(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/capture", post(capture_ocr_and_send))
        .with_state(state)
}

async fn capture_ocr_and_send(
    State(state): State<Arc<AppState>>,
) -> Result<Json<TextReceipt>, ApiError> {
    let frame = capture::capture_selected_region(&state.config.capture)
        .await
        .map_err(|error| {
            let message = error.to_string();
            if message.contains("selection cancelled") {
                ApiError::conflict(message)
            } else {
                ApiError::internal(message)
            }
        })?;

    let started = Instant::now();
    let text = ocr::extract_text(&state.config.ocr, Arc::new(frame.png))
        .await
        .map_err(|error| ApiError::internal(error.to_string()))?;
    let local_ocr_ms = started.elapsed().as_millis();

    info!(
        geometry = %frame.geometry,
        width = frame.width,
        height = frame.height,
        chars = text.chars().count(),
        local_ocr_ms,
        receiver = %state.config.server.url,
        "captured selected region, ran local OCR, sending text"
    );

    let submission = TextSubmission {
        device_id: state.config.capture.device_id.clone(),
        text,
        image_sha256: frame.sha256,
        monitor_name: format!("selected-region:{}", frame.geometry),
        width: frame.width,
        height: frame.height,
        local_ocr_ms,
    };

    let response = state
        .client
        .send(&submission)
        .await
        .map_err(|error| ApiError::bad_gateway(error.to_string()))?;

    Ok(Json(response))
}

#[derive(Debug)]
struct ApiError {
    status: StatusCode,
    message: String,
}

impl ApiError {
    fn internal(message: impl Into<String>) -> Self {
        Self { status: StatusCode::INTERNAL_SERVER_ERROR, message: message.into() }
    }

    fn conflict(message: impl Into<String>) -> Self {
        Self { status: StatusCode::CONFLICT, message: message.into() }
    }

    fn bad_gateway(message: impl Into<String>) -> Self {
        Self { status: StatusCode::BAD_GATEWAY, message: message.into() }
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        tracing::error!(status = %self.status, error = %self.message, "capture request failed");
        (self.status, Json(ErrorResponse { error: self.message })).into_response()
    }
}
