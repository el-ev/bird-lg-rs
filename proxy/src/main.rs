use std::{net::SocketAddr, sync::Arc};

use axum::{
    Router,
    extract::Extension,
    routing::{get, post},
};
use clap::Command;
use config::Config;
use tokio::net::TcpListener;

use crate::middleware::auth::auth_middleware;

mod config;
mod handlers;
mod middleware;
mod services;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let matches = Command::new("bird-lg-proxy-rs")
        .version(env!("CARGO_PKG_VERSION"))
        .about("A proxy for bird control socket with additional features")
        .arg(
            clap::Arg::new("config")
                .short('c')
                .long("config")
                .value_name("FILE")
                .help("Sets a custom config file.")
                .default_value("config.json"),
        )
        .get_matches();

    let config_path = matches.get_one::<String>("config").unwrap();
    println!("Using config file: {}", config_path);
    let config = Arc::new(Config::new(config_path));

    let listener = TcpListener::bind(&config.listen).await?;
    let app = Router::new()
        .route("/bird", post(handlers::bird::handler))
        .route("/traceroute", get(handlers::traceroute::traceroute))
        .route("/traceroute4", get(handlers::traceroute::traceroute4))
        .route("/traceroute6", get(handlers::traceroute::traceroute6))
        .layer(axum::middleware::from_fn(auth_middleware))
        .layer(Extension(config));

    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await
}
