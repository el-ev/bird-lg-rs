mod cli;
mod config;
mod handlers;
mod services;
mod state;
mod utils;

use std::{net::SocketAddr, sync::Arc};

use crate::{
    cli::Cli,
    config::Config,
    handlers::{info, protocol, route, status, traceroute, ws},
    services::poller,
    state::AppState,
};

use axum::{
    Extension, Router,
    extract::Request,
    middleware::{self, Next},
    response::Response,
    routing::get,
};
use tokio::net::TcpListener;
use tower_http::cors::CorsLayer;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let cli = Cli::parse_args();

    let config = Config::load(&cli.config)
        .map_err(|e| anyhow::anyhow!("Failed to load config from {}: {}", cli.config, e))?;
    let config = Arc::new(config);

    let state = AppState::new();
    poller::spawn(state.clone(), config.clone());

    let app = Router::new()
        .route("/api/protocols", get(status::get_all_protocols))
        .route(
            "/api/protocols/{node_name}",
            get(status::get_node_protocols),
        )
        .route(
            "/api/protocols/{node_name}/{protocol}",
            get(protocol::get_protocol_details),
        )
        .route(
            "/api/traceroute/{node_name}",
            get(traceroute::proxy_traceroute),
        )
        .route("/api/routes/{node_name}", get(route::get_route))
        .route("/api/info", get(info::get_network_info))
        .route(
            "/api/info/port/{port}",
            get(info::get_network_info_with_port),
        )
        .route("/api/peering/{node_name}", get(info::get_node_peering))
        .route("/api/ws", get(ws::ws_handler))
        .layer(CorsLayer::permissive())
        .layer(middleware::from_fn(track_request))
        .layer(Extension(state))
        .layer(Extension(config.clone()));

    let mut handles = Vec::new();
    for listen_addr in &config.listen {
        let app_clone = app.clone();
        let addr = listen_addr.clone();

        let handle = tokio::spawn(async move {
            match TcpListener::bind(&addr).await {
                Ok(listener) => {
                    tracing::info!("Server listening on {}", addr);
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

async fn track_request(
    Extension(state): Extension<AppState>,
    request: Request,
    next: Next,
) -> Response {
    state.record_request();
    next.run(request).await
}
