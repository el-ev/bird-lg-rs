mod cli;
mod config;
mod handlers;
mod parser;
mod services;
mod state;

use std::sync::Arc;

use axum::{Extension, Router, routing::get};
use tower_http::cors::CorsLayer;

use crate::{
    cli::Cli,
    config::Config,
    handlers::{protocol, route, status, traceroute},
    services::poller,
    state::AppState,
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let cli = Cli::parse_args();

    let config = Config::load(&cli.config)
        .map_err(|e| anyhow::anyhow!("Failed to load config from {}: {}", cli.config, e))?;
    let config = Arc::new(config);
    let listen_addr = config.listen.clone();

    let state = AppState::new();
    poller::spawn(state.clone(), config.clone());

    let app = Router::new()
        .route("/api/status", get(status::get_status))
        .route(
            "/api/node/{node_name}/protocol/{protocol}",
            get(protocol::get_protocol_details),
        )
        .route("/api/traceroute", get(traceroute::proxy_traceroute))
        .route("/api/route", get(route::get_route))
        .layer(CorsLayer::permissive())
        .layer(Extension(state))
        .layer(Extension(config));

    let listener = tokio::net::TcpListener::bind(&listen_addr).await?;
    println!("Server listening on {}", listen_addr);
    axum::serve(listener, app).await?;

    Ok(())
}
