use std::sync::Arc;

use axum::{
    Extension,
    extract::Query,
    response::{IntoResponse, Response},
};
use common::utils::validate_target;
use serde::Deserialize;
use tracing::{error, info, warn};

use crate::{
    config::Config,
    services::{ping::build_ping_command, traceroute::IpVersion},
};

#[derive(Deserialize)]
pub struct PingQuery {
    target: String,
}

pub async fn ping(
    Extension(config): Extension<Arc<Config>>,
    Query(params): Query<PingQuery>,
) -> Response {
    run_ping(config, params, IpVersion::Any).await
}

pub async fn ping4(
    Extension(config): Extension<Arc<Config>>,
    Query(params): Query<PingQuery>,
) -> Response {
    run_ping(config, params, IpVersion::V4).await
}

pub async fn ping6(
    Extension(config): Extension<Arc<Config>>,
    Query(params): Query<PingQuery>,
) -> Response {
    run_ping(config, params, IpVersion::V6).await
}

async fn run_ping(config: Arc<Config>, params: PingQuery, version: IpVersion) -> Response {
    let target = params.target.trim().to_string();
    if let Err(e) = validate_target(&target) {
        warn!(%target, "Invalid ping target: {}", e);
        return (
            axum::http::StatusCode::BAD_REQUEST,
            format!("Invalid target: {}", e),
        )
            .into_response();
    }

    let mut cmd = match build_ping_command(&config, &target, version) {
        Some(cmd) => cmd,
        None => {
            error!("Ping requested but ping_bin not configured");
            return (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                "ping not configured",
            )
                .into_response();
        }
    };

    info!(%target, version = ?version, "Executing ping");
    match cmd.spawn() {
        Ok(child) => crate::utils::stream_command_output(child, "ping", target).await,
        Err(e) => {
            error!(error = %e, %target, "Failed to execute ping command");
            (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to execute ping: {}", e),
            )
                .into_response()
        }
    }
}
