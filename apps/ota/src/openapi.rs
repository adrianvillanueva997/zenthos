use utoipa::OpenApi;

use crate::routes::health;

// Define API documentation tags
const HEALTH_TAG: &str = "health";

// Define the OpenAPI document
#[derive(OpenApi)]
#[openapi(
    paths(health::r_health),
    components(schemas(health::HealthResponse)),
    tags(
        (name = HEALTH_TAG, description = "Health check endpoints")
    )
)]
pub struct ApiDoc;
