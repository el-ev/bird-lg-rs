use std::sync::Arc;

use axum::{
    body::Body,
    extract::{Extension, Path, Query},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use common::{models::TracerouteParams, validate_target};
use tracing::warn;

use crate::{config::Config, services::request::get_stream, state::AppState};

pub async fn proxy_traceroute(
    Query(params): Query<TracerouteParams>,
    Path(node_name): Path<String>,
    Extension(config): Extension<Arc<Config>>,
    Extension(state): Extension<AppState>,
) -> Response {
    let TracerouteParams { target, version } = params;
    let target = target.trim().to_string();

    if let Err(msg) = validate_target(&target) {
        return (StatusCode::BAD_REQUEST, msg).into_response();
    }

    if let Some(node_config) = config.nodes.iter().find(|n| n.name == node_name) {
        let endpoint = match version.as_str() {
            "4" => "traceroute4",
            "6" => "traceroute6",
            _ => "traceroute",
        };
        let endpoint_with_query = format!("/{}?target={}", endpoint, target);
        match get_stream(&state.http_client, node_config, &endpoint_with_query).await {
            Ok(stream) => Body::from_stream(stream).into_response(),
            Err(err_msg) => {
                warn!(
                    node = %node_name,
                    target = %target,
                    error = %err_msg,
                    "Failed to fetch traceroute information"
                );
                (StatusCode::BAD_GATEWAY, err_msg).into_response()
            }
        }
    } else {
        (StatusCode::NOT_FOUND, "Node not found").into_response()
    }
}
