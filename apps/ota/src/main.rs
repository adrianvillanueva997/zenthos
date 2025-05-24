use metrics::create_opentelemetry_layer;
use router::create_router;
use tracing::info;

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

#[tokio::main]
async fn main() {
    create_opentelemetry_layer();
    let app = create_router();
    // let app = Router::new()
    //     .layer(TraceLayer::new_for_http())
    //     .route("/users", post(create_user));
    info!("patata");
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
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
