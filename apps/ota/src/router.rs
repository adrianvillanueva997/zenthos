use crate::{
    openapi::ApiDoc,
    routes::v1::{firmware::r_firmware, health::r_health},
};
use axum::{Router, routing::get};
use tower_http::trace::TraceLayer;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

pub fn create_router() -> Router {
    let router = Router::new()
        .route("/api/v1/health", get(r_health))
        .route("/api/v1/firmware", get(r_firmware))
        .layer(TraceLayer::new_for_http());

    router.merge(SwaggerUi::new("/docs").url("/api-docs/openapi.json", ApiDoc::openapi()))
}
