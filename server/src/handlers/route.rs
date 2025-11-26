use std::{net::IpAddr, sync::Arc};

use axum::{
    body::Body,
    extract::{Extension, Path, Query},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use ipnet::IpNet;
use serde::Deserialize;
use tracing::warn;

use crate::{config::Config, services::request::post_stream, state::AppState};

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

        let command = if params.all {
            format!("show route for {} all", params.target)
        } else {
            format!("show route for {}", params.target)
        };

        match post_stream(&state.http_client, node, "/bird", &command).await {
            Ok(stream) => Body::from_stream(stream).into_response(),
            Err(err_msg) => {
                warn!(
                    node = %node_name,
                    target = %params.target,
                    error = %err_msg,
                    "Failed to fetch route information"
                );
                (StatusCode::BAD_GATEWAY, err_msg).into_response()
            }
        }
    } else {
        (StatusCode::NOT_FOUND, "Node not found").into_response()
    }
}
