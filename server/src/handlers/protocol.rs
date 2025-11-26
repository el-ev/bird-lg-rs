use std::{io, sync::Arc};

use axum::{
    body::Body,
    extract::{Extension, Path},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use futures_util::StreamExt;
use tracing::warn;

use crate::{config::Config, state::AppState};

pub async fn get_protocol_details(
    Path((node_name, protocol)): Path<(String, String)>,
    Extension(config): Extension<Arc<Config>>,
    Extension(state): Extension<AppState>,
) -> Response {
    if let Some(node) = config.nodes.iter().find(|n| n.name == node_name) {
        let url = format!("{}/bird", node.url);

        let mut req = state
            .http_client
            .post(&url)
            .body(format!("show protocols all {}", protocol));

        if let Some(secret) = &node.shared_secret {
            req = req.header("x-shared-secret", secret);
        }

        match req.send().await {
            Ok(resp) => {
                let status = resp.status();
                if !status.is_success() {
                    warn!(
                        node = %node_name,
                        protocol = %protocol,
                        status = %status,
                        "Node returned non-success status for protocol details"
                    );
                    return (
                        StatusCode::BAD_GATEWAY,
                        "Node rejected protocol details request",
                    )
                        .into_response();
                }

                let stream = resp
                    .bytes_stream()
                    .map(|chunk| chunk.map_err(io::Error::other));

                Body::from_stream(stream).into_response()
            }
            Err(e) => {
                warn!(node = %node_name, error = ?e, "Failed to fetch protocol details");
                (
                    StatusCode::BAD_GATEWAY,
                    "Unable to reach the node at the moment. Please check back soon.",
                )
                    .into_response()
            }
        }
    } else {
        (StatusCode::NOT_FOUND, "Node not found").into_response()
    }
}
