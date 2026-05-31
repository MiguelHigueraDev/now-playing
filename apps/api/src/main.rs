mod auth;
mod colors;
mod error;
mod handlers;
mod routes;
mod state;
mod svg;

use std::net::SocketAddr;

use axum::Router;
use dotenvy::dotenv;
use tokio::net::TcpListener;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

use crate::routes::create_router;
use crate::state::AppState;

#[tokio::main]
async fn main() {
    dotenv().ok();

    tracing_subscriber::registry()
        .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| "info,api=debug".into()))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let host = std::env::var("API_HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
    let port: u16 = std::env::var("API_PORT")
        .or_else(|_| std::env::var("PORT"))
        .ok()
        .and_then(|value| value.parse().ok())
        .unwrap_or(3000);

    let auth_token = std::env::var("NOW_PLAYING_TOKEN")
        .expect("NOW_PLAYING_TOKEN must be set in the environment");

    let state = AppState::new(auth_token);
    let app = build_app(state);

    let addr: SocketAddr = format!("{host}:{port}")
        .parse()
        .expect("invalid API bind address");

    let listener = TcpListener::bind(addr)
        .await
        .expect("failed to bind API server");

    info!(%addr, "now-playing API listening");

    axum::serve(listener, app)
        .await
        .expect("API server failed");
}

fn build_app(state: AppState) -> Router {
    create_router(state).layer((
        TraceLayer::new_for_http(),
        CorsLayer::new()
            .allow_origin(Any)
            .allow_methods(Any)
            .allow_headers(Any),
    ))
}
