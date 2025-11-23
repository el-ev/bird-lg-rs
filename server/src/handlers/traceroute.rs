use std::{io, sync::Arc};

use axum::{
    body::Body,
    extract::{Extension, Query},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use futures_util::StreamExt;
use serde::Deserialize;
use tracing::warn;

use crate::{config::Config, services::traceroute::validate_target};

#[derive(Deserialize)]
pub struct TracerouteParams {
    pub node: String,
    pub target: String,
    #[serde(default)]
    pub version: String,
}

pub async fn proxy_traceroute(
    Query(params): Query<TracerouteParams>,
    Extension(config): Extension<Arc<Config>>,
) -> Response {
    let TracerouteParams {
        node,
        target,
        version,
    } = params;
    let target = target.trim().to_string();

    if let Err(msg) = validate_target(&target) {
        return (StatusCode::BAD_REQUEST, msg).into_response();
    }

    if let Some(node_config) = config.nodes.iter().find(|n| n.name == node) {
        let client = reqwest::Client::new();
        let endpoint = match version.as_str() {
            "4" => "traceroute4",
            "6" => "traceroute6",
            _ => "traceroute",
        };
        let url = format!("{}/{}?target={}", node_config.url, endpoint, target);

        match client.get(&url).send().await {
            Ok(resp) => {
                let status = resp.status();
                if !status.is_success() {
                    warn!(
                        node = %node,
                        status = %status,
                        "Node returned non-success status for traceroute"
                    );
                    return (
                        StatusCode::BAD_GATEWAY,
                        "Traceroute service on node is currently unavailable",
                    )
                        .into_response();
                }

                let stream = resp
                    .bytes_stream()
                    .map(|chunk| chunk.map_err(io::Error::other));

                Body::from_stream(stream).into_response()
            }
            Err(e) => {
                warn!(node = %node, error = ?e, "Failed to start traceroute");
                (
                    StatusCode::BAD_GATEWAY,
                    "Node is unreachable, traceroute could not be started.",
                )
                    .into_response()
            }
        }
    } else {
        (StatusCode::NOT_FOUND, "Node not found").into_response()
    }
}
