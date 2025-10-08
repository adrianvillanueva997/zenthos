use axum::{
    http::StatusCode,
    response::{IntoResponse, Json},
};
use serde::Serialize;
use tracing::{info, instrument};
use utoipa::ToSchema;

#[derive(Serialize, ToSchema)]
pub struct HealthResponse {
    status: &'static str,
    version: &'static str,
}
/// Check service health status
///
/// Returns information about the service health and version.
#[utoipa::path(
    get,
    path = "/health",
    responses(
        (status = 200, description = "Service is healthy", body = HealthResponse)
    )
)]
#[instrument]
pub async fn r_health() -> impl IntoResponse {
    let health = HealthResponse {
        status: "healthy",
        version: env!("CARGO_PKG_VERSION"),
    };
    info!("Health check successful. Version: {}", health.version);
    (StatusCode::OK, Json(health))
}
