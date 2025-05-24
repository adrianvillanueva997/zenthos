use crate::{openapi::ApiDoc, routes::health::r_health};
use axum::{Router, routing::get};
use tower_http::trace::TraceLayer;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

pub fn create_router() -> Router {
    let router = Router::new()
        .route("/health", get(r_health))
        .layer(TraceLayer::new_for_http());

    router.merge(SwaggerUi::new("/docs").url("/api-docs/openapi.json", ApiDoc::openapi()))
}
