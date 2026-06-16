use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextSubmission {
    pub device_id: String,
    pub text: String,
    pub image_sha256: String,
    pub monitor_name: String,
    pub width: u32,
    pub height: u32,
    pub local_ocr_ms: u128,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextReceipt {
    pub request_id: Uuid,
    pub device_id: String,
    pub text: String,
    pub received_chars: usize,
    pub image_sha256: String,
    pub text_saved_to: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorResponse {
    pub error: String,
}
