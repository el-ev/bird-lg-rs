use axum::{
    Json,
    extract::{Extension, Path},
    http::StatusCode,
    response::{IntoResponse, Response},
};

use crate::state::{AppResponse, AppState};

pub async fn get_all_protocols(Extension(state): Extension<AppState>) -> Json<AppResponse> {
    let nodes = state.nodes.read().unwrap().clone();
    Json(AppResponse::Protocols { data: nodes })
}

pub async fn get_node_protocols(
    Path(node_name): Path<String>,
    Extension(state): Extension<AppState>,
) -> Response {
    let nodes = state.nodes.read().unwrap();

    if let Some(node) = nodes.iter().find(|n| n.name == node_name) {
        Json(node.clone()).into_response()
    } else {
        (StatusCode::NOT_FOUND, "Node not found").into_response()
    }
}
