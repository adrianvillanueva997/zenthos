use axum::{
    body::Bytes,
    http::{StatusCode, header},
    response::{IntoResponse, Response},
};
use tracing::instrument;

/// Get firmware binary
///
/// Returns the firmware binary file for OTA updates.
#[utoipa::path(
    get,
    path = "/firmware",
    responses(
        (status = 200, description = "Firmware binary", content_type = "application/octet-stream")
    )
)]
#[instrument]
pub async fn r_firmware() -> impl IntoResponse {
    let firmware_data = get_firmware_data().await;
    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "application/octet-stream")
        .header(
            header::CONTENT_DISPOSITION,
            "attachment; filename=\"firmware.bin\"",
        )
        .body(axum::body::Body::from(firmware_data))
        .unwrap()
}

// Helper function to get firmware data
async fn get_firmware_data() -> Bytes {
    Bytes::from(vec![1, 2, 3, 4])
}
