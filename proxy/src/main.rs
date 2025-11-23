use std::{net::SocketAddr, sync::Arc};

use axum::{
    Router,
    extract::Extension,
    routing::{get, post},
};
use config::Config;
use tokio::net::TcpListener;
use tower_http::cors::CorsLayer;

use crate::{cli::Cli, middleware::auth::auth_middleware};

mod cli;
mod config;
mod handlers;
mod middleware;
mod services;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let cli = Cli::parse_args();

    let config_path = &cli.config;
    println!("Using config file: {}", config_path);
    let config = Arc::new(Config::new(config_path));

    let listener = TcpListener::bind(&config.listen).await?;
    let app = Router::new()
        .route("/bird", post(handlers::bird::handler))
        .route("/traceroute", get(handlers::traceroute::traceroute))
        .route("/traceroute4", get(handlers::traceroute::traceroute4))
        .route("/traceroute6", get(handlers::traceroute::traceroute6))
        .layer(CorsLayer::permissive())
        .layer(axum::middleware::from_fn(auth_middleware))
        .layer(Extension(config));

    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await
}
