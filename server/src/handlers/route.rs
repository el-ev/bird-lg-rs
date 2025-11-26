use std::{io, net::IpAddr, sync::Arc};

use axum::{
    body::Body,
    extract::{Extension, Path, Query},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use futures_util::StreamExt;
use ipnet::IpNet;
use serde::Deserialize;
use tracing::warn;

use crate::{config::Config, state::AppState};

#[derive(Deserialize)]
pub struct RouteParams {
    target: String,
    #[serde(default)]
    all: bool,
}

pub async fn get_route(
    Path(node_name): Path<String>,
    Query(params): Query<RouteParams>,
    Extension(config): Extension<Arc<Config>>,
    Extension(state): Extension<AppState>,
) -> Response {
    if let Some(node) = config.nodes.iter().find(|n| n.name == node_name) {
        let target = params.target.trim();
        let is_valid_target = target.parse::<IpAddr>().is_ok() || target.parse::<IpNet>().is_ok();

        if !is_valid_target {
            return (
                StatusCode::BAD_REQUEST,
                "Invalid target format (must be IP or CIDR)",
            )
                .into_response();
        }

        let url = format!("{}/bird", node.url);

        let command = if params.all {
            format!("show route for {} all", params.target)
        } else {
            format!("show route for {}", params.target)
        };

        let mut req = state.http_client.post(&url).body(command);
        if let Some(secret) = &node.shared_secret {
            req = req.header("x-shared-secret", secret);
        }

        match req.send().await {
            Ok(resp) => {
                let status = resp.status();
                if !status.is_success() {
                    warn!(
                        node = %node_name,
                        target = %params.target,
                        status = %status,
                        "Node returned non-success status for route request"
                    );
                    return (StatusCode::BAD_GATEWAY, "Node rejected route request")
                        .into_response();
                }

                let stream = resp
                    .bytes_stream()
                    .map(|chunk| chunk.map_err(io::Error::other));

                Body::from_stream(stream).into_response()
            }
            Err(e) => {
                warn!(
                    node = %node_name,
                    error = %e,
                    "Failed to contact node"
                );
                (StatusCode::BAD_GATEWAY, "Failed to contact node").into_response()
            }
        }
    } else {
        (StatusCode::NOT_FOUND, "Node not found").into_response()
    }
}
