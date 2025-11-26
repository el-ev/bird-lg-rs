use std::sync::Arc;

use axum::{
    body::Body,
    extract::{Extension, Path},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use tracing::warn;

use crate::{config::Config, services::request::post_stream, state::AppState};

pub async fn get_protocol_details(
    Path((node_name, protocol)): Path<(String, String)>,
    Extension(config): Extension<Arc<Config>>,
    Extension(state): Extension<AppState>,
) -> Response {
    if let Some(node) = config.nodes.iter().find(|n| n.name == node_name) {
        let command = format!("show protocols all {}", protocol);

        match post_stream(&state.http_client, node, "/bird", &command).await {
            Ok(stream) => Body::from_stream(stream).into_response(),
            Err(err_msg) => {
                warn!(
                    node = %node_name,
                    protocol = %protocol,
                    error = %err_msg,
                    "Failed to fetch protocol details"
                );
                (StatusCode::BAD_GATEWAY, err_msg).into_response()
            }
        }
    } else {
        (StatusCode::NOT_FOUND, "Node not found").into_response()
    }
}
