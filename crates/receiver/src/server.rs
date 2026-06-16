use std::{sync::Arc, time::Instant};

use axum::{
    Json, Router,
    body::Bytes,
    extract::{DefaultBodyLimit, State},
    http::{HeaderMap, StatusCode, header},
    response::{IntoResponse, Response},
    routing::{get, post},
};
use screen_ocr_common::{
    DEVICE_ID_HEADER, ErrorResponse, IMAGE_SHA256_HEADER, OcrResponse,
};
use serde_json::{Value, json};
use tracing::{error, info};
use uuid::Uuid;

use crate::{config::ReceiverConfig, ocr, storage};

#[derive(Clone)]
pub struct AppState {
    pub config: ReceiverConfig,
}

pub fn router(state: Arc<AppState>) -> Router {
    let max_bytes = state.config.server.max_upload_mb * 1024 * 1024;

    Router::new()
        .route("/healthz", get(healthz))
        .route("/v1/ocr", post(run_ocr))
        .layer(DefaultBodyLimit::max(max_bytes))
        .with_state(state)
}

async fn healthz() -> Json<Value> {
    Json(json!({"status": "ok"}))
}

async fn run_ocr(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<Json<OcrResponse>, ApiError> {
    authorize(&headers, &state.config.server.token)?;

    if body.is_empty() {
        return Err(ApiError::bad_request("request body is empty"));
    }

    let mime = content_type(&headers)?;
    let device_id = header_string(&headers, DEVICE_ID_HEADER).unwrap_or_else(|| "unknown".to_owned());
    let image_sha256 = header_string(&headers, IMAGE_SHA256_HEADER);
    let request_id = Uuid::new_v4();
    let started = Instant::now();

    info!(
        %request_id,
        %device_id,
        mime,
        bytes = body.len(),
        "received screenshot"
    );

    let text = ocr::extract_text(&state.config.ocr, &body)
        .await
        .map_err(|error| {
            error!(%request_id, %error, "OCR failed");
            ApiError::internal(error.to_string())
        })?;

    let stored = storage::store(
        &state.config.storage,
        request_id,
        mime,
        &body,
        &text,
    )
    .await
    .map_err(|error| {
        error!(%request_id, %error, "failed to store OCR artifacts");
        ApiError::internal(error.to_string())
    })?;

    Ok(Json(OcrResponse {
        request_id,
        device_id,
        text,
        elapsed_ms: started.elapsed().as_millis(),
        image_sha256,
        image_saved_to: stored.image,
        text_saved_to: stored.text,
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

fn content_type(headers: &HeaderMap) -> Result<&'static str, ApiError> {
    let value = headers
        .get(header::CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .unwrap_or_default()
        .split(';')
        .next()
        .unwrap_or_default()
        .trim();

    match value {
        "image/png" => Ok("image/png"),
        "image/jpeg" => Ok("image/jpeg"),
        _ => Err(ApiError::unsupported_media(
            "content-type must be image/png or image/jpeg",
        )),
    }
}

fn header_string(headers: &HeaderMap, name: &'static str) -> Option<String> {
    headers
        .get(name)
        .and_then(|value| value.to_str().ok())
        .map(ToOwned::to_owned)
}

#[derive(Debug)]
struct ApiError {
    status: StatusCode,
    message: String,
}

impl ApiError {
    fn bad_request(message: impl Into<String>) -> Self {
        Self {
            status: StatusCode::BAD_REQUEST,
            message: message.into(),
        }
    }

    fn unauthorized(message: impl Into<String>) -> Self {
        Self {
            status: StatusCode::UNAUTHORIZED,
            message: message.into(),
        }
    }

    fn unsupported_media(message: impl Into<String>) -> Self {
        Self {
            status: StatusCode::UNSUPPORTED_MEDIA_TYPE,
            message: message.into(),
        }
    }

    fn internal(message: impl Into<String>) -> Self {
        Self {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            message: message.into(),
        }
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        (
            self.status,
            Json(ErrorResponse {
                error: self.message,
            }),
        )
            .into_response()
    }
}
