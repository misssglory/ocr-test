use std::sync::Arc;

use axum::{
    Json, Router,
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::post,
};
use screen_ocr_common::{ErrorResponse, OcrResponse};
use tracing::info;

use crate::{capture, client::OcrClient, config::SenderConfig};

#[derive(Clone)]
pub struct AppState {
    pub config: SenderConfig,
    pub client: OcrClient,
}

pub fn router(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/capture", post(capture_and_send))
        .with_state(state)
}

async fn capture_and_send(
    State(state): State<Arc<AppState>>,
) -> Result<Json<OcrResponse>, ApiError> {
    let frame = tokio::task::spawn_blocking(capture::capture_focused_window)
        .await
        .map_err(|error| ApiError::internal(format!("capture task failed: {error}")))?
        .map_err(|error| ApiError::internal(error.to_string()))?;

    info!(
        app = %frame.app_name,
        title = %frame.window_title,
        width = frame.width,
        height = frame.height,
        bytes = frame.bytes.len(),
        sha256 = %frame.sha256,
        receiver = %state.config.server.url,
        "captured focused window; sending to receiver"
    );

    let response = state
        .client
        .send(frame)
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
        Self {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            message: message.into(),
        }
    }

    fn bad_gateway(message: impl Into<String>) -> Self {
        Self {
            status: StatusCode::BAD_GATEWAY,
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
