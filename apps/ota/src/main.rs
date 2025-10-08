use std::env;

use metrics::create_opentelemetry_layer;
use router::create_router;
use tracing::{error, info, warn};
mod auth;

mod db;
mod events;
mod metrics;
mod middlewares;
mod models;
mod openapi;
mod router;
mod routes;
mod services;
mod utils;

#[tokio::main]
async fn main() {
    create_opentelemetry_layer();
    let app = create_router();
    let port: String;
    if let Ok(env_port) = env::var("PORT") {
        port = env_port;
        info!("Port defined in environment variable.");
    } else {
        port = utils::PORT.to_string();
        warn!("Port not defined in environment variable, using the default port: {port}");
    }
    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", &port)).await;
    match listener {
        Ok(listener) => {
            info!("Starting server on port: {} ", &port);
            axum::serve(listener, app).await.unwrap();
        }
        Err(e) => {
            error!("Could not start the server: {e}")
        }
    }
    //
}

// async fn create_user(Json(payload): Json<CreateUser>) -> (StatusCode, Json<User>) {
//     let user = User {
//         id: 1337,
//         username: payload.username,
//     };

//     (StatusCode::CREATED, Json(user))
// }

// #[derive(Deserialize)]
// struct CreateUser {
//     username: String,
// }

// #[derive(Serialize)]
// struct User {
//     id: u64,
//     username: String,
// }
