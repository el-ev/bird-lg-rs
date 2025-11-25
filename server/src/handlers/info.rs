use axum::{
    Json,
    extract::{Extension, Path},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use std::sync::Arc;

use crate::config::{Config, NetworkInfo};
use crate::state::AppState;

pub async fn get_network_info(
    Extension(config): Extension<Arc<Config>>,
) -> Json<Option<NetworkInfo>> {
    Json(config.network.clone())
}

pub async fn get_node_peering(
    Path(node_name): Path<String>,
    Extension(state): Extension<AppState>,
) -> Response {
    // Get peering info from state (fetched from proxy)
    let nodes = state.nodes.read().unwrap();

    if let Some(node) = nodes.iter().find(|n| n.name == node_name) {
        Json(node.peering.clone()).into_response()
    } else {
        (StatusCode::NOT_FOUND, "Node not found").into_response()
    }
}
