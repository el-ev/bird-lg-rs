use std::sync::Arc;

use axum::{Extension, extract::Query, response::Response};
use serde::Deserialize;

use crate::{
    config::Config,
    services::{command_runner::run_command, ping::build_ping_command, traceroute::IpVersion},
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
    run_command(
        config,
        params.target,
        version,
        "ping",
        "ping not configured",
        build_ping_command,
    )
    .await
}
