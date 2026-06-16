use std::sync::Arc;

use axum::{
    Json, Router,
    extract::{DefaultBodyLimit, State},
    http::{HeaderMap, StatusCode, header},
    response::{IntoResponse, Response},
    routing::{get, post},
};
use screen_ocr_common::{ErrorResponse, TextReceipt, TextSubmission};
use serde_json::{Value, json};
use tracing::info;
use uuid::Uuid;

use crate::{config::ReceiverConfig, storage};

#[derive(Clone)]
pub struct AppState {
    pub config: ReceiverConfig,
}

pub fn router(state: Arc<AppState>) -> Router {
    let max_bytes = state.config.server.max_text_kb * 1024;

    Router::new()
        .route("/healthz", get(healthz))
        .route("/v1/text", post(receive_text))
        .layer(DefaultBodyLimit::max(max_bytes))
        .with_state(state)
}

async fn healthz() -> Json<Value> {
    Json(json!({"status": "ok"}))
}

async fn receive_text(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(submission): Json<TextSubmission>,
) -> Result<Json<TextReceipt>, ApiError> {
    authorize(&headers, &state.config.server.token)?;

    if submission.device_id.trim().is_empty() {
        return Err(ApiError::bad_request("device_id is empty"));
    }

    let request_id = Uuid::new_v4();
    let received_chars = submission.text.chars().count();

    info!(
        %request_id,
        device_id = %submission.device_id,
        monitor = %submission.monitor_name,
        width = submission.width,
        height = submission.height,
        received_chars,
        local_ocr_ms = submission.local_ocr_ms,
        "received locally recognized text"
    );

    let text_saved_to = storage::store_text(
        &state.config.storage,
        request_id,
        &submission.text,
    )
    .await
    .map_err(|error| ApiError::internal(error.to_string()))?;

    Ok(Json(TextReceipt {
        request_id,
        device_id: submission.device_id,
        text: submission.text,
        received_chars,
        image_sha256: submission.image_sha256,
        text_saved_to,
    }))
}

fn authorize(headers: &HeaderMap, expected_token: &str) -> Result<(), ApiError> {
    let supplied = headers
        .get(header::AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.strip_prefix("Bearer "));

    if supplied == Some(expected_token) {
        Ok(())
    } else {
        Err(ApiError::unauthorized("missing or invalid bearer token"))
    }
}

#[derive(Debug)]
struct ApiError {
    status: StatusCode,
    message: String,
}

impl ApiError {
    fn bad_request(message: impl Into<String>) -> Self {
        Self { status: StatusCode::BAD_REQUEST, message: message.into() }
    }
    fn unauthorized(message: impl Into<String>) -> Self {
        Self { status: StatusCode::UNAUTHORIZED, message: message.into() }
    }
    fn internal(message: impl Into<String>) -> Self {
        Self { status: StatusCode::INTERNAL_SERVER_ERROR, message: message.into() }
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        (self.status, Json(ErrorResponse { error: self.message })).into_response()
    }
}
