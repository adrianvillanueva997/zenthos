use axum::{
    http::StatusCode,
    response::{IntoResponse, Json},
};
use serde::Serialize;

#[derive(Serialize)]
struct HealthResponse {
    status: &'static str,
    version: &'static str,
}

pub async fn r_health() -> impl IntoResponse {
    let health = HealthResponse {
        status: "healthy",
        version: env!("CARGO_PKG_VERSION"),
    };

    (StatusCode::OK, Json(health))
}
