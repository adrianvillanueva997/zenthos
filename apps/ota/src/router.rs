use crate::routes::health::{self, r_health};
use axum::{Router, routing::get};
use tower_http::trace::TraceLayer;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

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
struct ApiDoc;

pub fn create_router() -> Router {
    let router = Router::new()
        .route("/health", get(r_health))
        .layer(TraceLayer::new_for_http());

    router.merge(SwaggerUi::new("/docs").url("/api-docs/openapi.json", ApiDoc::openapi()))
}
