use utoipa::OpenApi;

use crate::routes::{firmware, health};

// Define API documentation tags
const HEALTH_TAG: &str = "health";
const FIRMWARE_TAG: &str = "firmware";

// Define the OpenAPI document
#[derive(OpenApi)]
#[openapi(
    paths(
        health::r_health,
        firmware::firmware
    ),
    components(schemas(
        health::HealthResponse
    )),
    tags(
        (name = HEALTH_TAG, description = "Health check endpoints"),
        (name = FIRMWARE_TAG, description = "Firmware update endpoints")
    )
)]
pub struct ApiDoc;
