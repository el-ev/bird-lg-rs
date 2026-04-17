use std::sync::Arc;

use axum::{Extension, extract::Query, response::Response};
use serde::Deserialize;

use crate::{
    config::Config,
    services::{
        command_runner::run_command,
        traceroute::{IpVersion, build_traceroute_command},
    },
};

#[derive(Deserialize)]
pub struct TracerouteQuery {
    target: String,
}

pub async fn traceroute(
    Extension(config): Extension<Arc<Config>>,
    Query(params): Query<TracerouteQuery>,
) -> Response {
    run_traceroute(config, params, IpVersion::Any).await
}

pub async fn traceroute4(
    Extension(config): Extension<Arc<Config>>,
    Query(params): Query<TracerouteQuery>,
) -> Response {
    run_traceroute(config, params, IpVersion::V4).await
}

pub async fn traceroute6(
    Extension(config): Extension<Arc<Config>>,
    Query(params): Query<TracerouteQuery>,
) -> Response {
    run_traceroute(config, params, IpVersion::V6).await
}

async fn run_traceroute(
    config: Arc<Config>,
    params: TracerouteQuery,
    version: IpVersion,
) -> Response {
    run_command(
        config,
        params.target,
        version,
        "traceroute",
        "traceroute not configured",
        build_traceroute_command,
    )
    .await
}
