use axum::{
    Json,
    extract::{Extension, Path},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use std::sync::Arc;

use crate::config::Config;
use crate::state::AppState;
use common::api::AppResponse;

pub async fn get_network_info(
    Extension(config): Extension<Arc<Config>>,
    Extension(state): Extension<AppState>,
) -> Json<AppResponse> {
    if let Some(network) = &config.network {
        let peering = state.peering.read().unwrap();
        let mut info = network.clone();
        info.peering = peering.clone();
        Json(AppResponse::NetworkInfo(info))
    } else {
        Json(AppResponse::Error("Network info not available".to_string()))
    }
}

pub async fn get_network_info_with_port(
    Path(port): Path<u16>,
    Extension(config): Extension<Arc<Config>>,
    Extension(state): Extension<AppState>,
) -> Response {
    if !(20000..=29999).contains(&port) {
        return (
            StatusCode::BAD_REQUEST,
            Json(AppResponse::Error(
                "Port must be between 20000 and 29999".to_string(),
            )),
        )
            .into_response();
    }

    if let Some(network) = &config.network {
        let peering = state.peering.read().unwrap();
        let mut info = network.clone();
        let mut modified_peering = peering.clone();

        for peer_info in modified_peering.values_mut() {
            if let Some(endpoint) = &mut peer_info.endpoint {
                *endpoint = format!("{}:{}", endpoint, port);
            }
        }
        info.peering = modified_peering;
        Json(AppResponse::NetworkInfo(info)).into_response()
    } else {
        Json(AppResponse::Error("Network info not available".to_string())).into_response()
    }
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
