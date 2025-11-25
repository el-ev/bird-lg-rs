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
    let peering = state.peering.read().unwrap();

    if let Some(info) = peering.get(&node_name) {
        Json(info.clone()).into_response()
    } else {
        (StatusCode::NOT_FOUND, "Node not found or no peering info").into_response()
    }
}
