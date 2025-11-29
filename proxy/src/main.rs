use std::{net::SocketAddr, sync::Arc};

use axum::{
    Router,
    extract::Extension,
    routing::{get, post},
};
use config::Config;
use tokio::net::TcpListener;
use tower_http::cors::CorsLayer;
use tracing::info;

use crate::{cli::Cli, middleware::auth::auth_middleware};

mod cli;
mod config;
mod handlers;
mod middleware;
mod services;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let cli = Cli::parse_args();

    let config_path = &cli.config;
    info!("Using config file: {}", config_path);
    let config = Arc::new(Config::new(config_path)?);

    let app = Router::new()
        .route("/bird", post(handlers::bird::handler))
        .route("/traceroute", get(handlers::traceroute::traceroute))
        .route("/traceroute4", get(handlers::traceroute::traceroute4))
        .route("/traceroute6", get(handlers::traceroute::traceroute6))
        .route("/peering", get(handlers::peering::get_peering_info))
        .layer(CorsLayer::permissive())
        .layer(axum::middleware::from_fn(auth_middleware))
        .layer(Extension(config.clone()));

    let mut handles = Vec::new();
    for listen_addr in &config.listen {
        let app_clone = app.clone();
        let addr = listen_addr.clone();

        let handle = tokio::spawn(async move {
            match TcpListener::bind(&addr).await {
                Ok(listener) => {
                    info!("Proxy listening on {}", addr);
                    if let Err(e) = axum::serve(
                        listener,
                        app_clone.into_make_service_with_connect_info::<SocketAddr>(),
                    )
                    .await
                    {
                        tracing::error!("Server on {} failed: {}", addr, e);
                        Err(anyhow::anyhow!("Server on {} failed: {}", addr, e))
                    } else {
                        Ok(())
                    }
                }
                Err(e) => {
                    tracing::error!("Failed to bind to {}: {}", addr, e);
                    Err(anyhow::anyhow!("Failed to bind to {}: {}", addr, e))
                }
            }
        });

        handles.push(handle);
    }

    let (result, _index, _remaining) = futures::future::select_all(handles).await;
    result??;

    Ok(())
}
