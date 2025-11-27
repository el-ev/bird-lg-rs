use std::sync::Arc;

use axum::extract::{Extension, Path};
use tracing::warn;

use crate::{
    config::Config,
    services::request::post_text,
    state::{AppState, AppResponse},
};
use axum::Json;

pub async fn get_protocol_details(
    Path((node_name, protocol)): Path<(String, String)>,
    Extension(config): Extension<Arc<Config>>,
    Extension(state): Extension<AppState>,
) -> Json<AppResponse> {
    if let Some(node) = config.nodes.iter().find(|n| n.name == node_name) {
        let command = format!("show protocols all {}", protocol);

        match post_text(&state.http_client, node, "/bird", &command).await {
            Ok(text) => Json(AppResponse::ProtocolDetailsResult {
                node: node_name,
                protocol,
                details: text,
            }),
            Err(err_msg) => {
                warn!(
                    node = %node_name,
                    protocol = %protocol,
                    error = %err_msg,
                    "Failed to fetch protocol details"
                );
                Json(AppResponse::Error(err_msg))
            }
        }
    } else {
        Json(AppResponse::Error("Node not found".into()))
    }
}
