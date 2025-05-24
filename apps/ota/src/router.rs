use axum::{Router, routing::get};
use tower_http::trace::TraceLayer;

use crate::routes::health::r_health;

pub fn create_router() -> Router {
    Router::new()
        .layer(TraceLayer::new_for_http())
        .route("/health", get(r_health))
}
