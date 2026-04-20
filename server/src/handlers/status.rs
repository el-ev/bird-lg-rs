use axum::{
    Json,
    extract::{Extension, Path},
    http::StatusCode,
    response::{IntoResponse, Response},
};

use crate::state::{AppResponse, AppState};

#[utoipa::path(
    get,
    path = "/api/protocols",
    tag = "protocols",
    responses(
        (
            status = 200,
            description = "Current protocol snapshots for all nodes as an AppResponse::Protocols payload",
            body = common::api::AppResponse
        )
    )
)]
pub async fn get_all_protocols(Extension(state): Extension<AppState>) -> Json<AppResponse> {
    let nodes = state.nodes.read().unwrap().clone();
    Json(AppResponse::Protocols { data: nodes })
}

#[utoipa::path(
    get,
    path = "/api/protocols/{node_name}",
    tag = "protocols",
    params(
        ("node_name" = String, Path, description = "Node name")
    ),
    responses(
        (
            status = 200,
            description = "Current protocol snapshot for the requested node",
            body = common::models::NodeProtocol
        ),
        (
            status = 404,
            description = "Node not found",
            body = String,
            content_type = "text/plain"
        )
    )
)]
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
