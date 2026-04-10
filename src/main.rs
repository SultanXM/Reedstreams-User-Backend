use axum::{
    routing::get,
    Router,
};
use std::net::SocketAddr;
use tower_http::cors::{Any, CorsLayer};

mod auth;
mod db;
mod models;
mod routes;
mod ws_state;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    dotenvy::dotenv().ok();

    let pool = db::init_db().await.expect("Failed to connect to database");

    let viewers = ws_state::ActiveViewers::new();

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(vec![
            "authorization".parse().unwrap(),
            "content-type".parse().unwrap(),
        ]);

    let ws_state = routes::AppState {
        pool: pool.clone(),
        viewers: viewers.clone(),
    };

    let app = Router::new()
        .route("/", get(health_check))
        .merge(routes::user_routes(pool.clone()))
        .merge(routes::profile_routes(pool.clone()))
        .merge(routes::playlist_routes(pool.clone()))
        .merge(routes::chat_routes(pool.clone()))
        .merge(routes::admin_routes(pool.clone()))
        .merge(routes::views_routes(pool.clone()))
        .merge(routes::default_source_routes(pool.clone()))
        .merge(routes::ws_views_routes(ws_state))
        .layer(cors);

    let addr = SocketAddr::from(([0, 0, 0, 0], 8080));
    tracing::info!("Server running on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn health_check() -> &'static str {
    "Reedstreams API is running!"
}
