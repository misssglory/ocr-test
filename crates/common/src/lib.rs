use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub const DEVICE_ID_HEADER: &str = "x-device-id";
pub const IMAGE_SHA256_HEADER: &str = "x-image-sha256";
pub const IMAGE_WIDTH_HEADER: &str = "x-image-width";
pub const IMAGE_HEIGHT_HEADER: &str = "x-image-height";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OcrResponse {
    pub request_id: Uuid,
    pub device_id: String,
    pub text: String,
    pub elapsed_ms: u128,
    pub image_sha256: Option<String>,
    pub image_saved_to: Option<String>,
    pub text_saved_to: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorResponse {
    pub error: String,
}
