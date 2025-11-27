use std::sync::Arc;

use axum::extract::{Extension, Path, Query};
use common::{models::TracerouteParams, validate_target};
use tracing::warn;

use crate::{
    config::Config,
    services::request::get_text,
    state::{AppResponse, AppState},
};
use axum::Json;

pub async fn proxy_traceroute(
    Query(params): Query<TracerouteParams>,
    Path(node_name): Path<String>,
    Extension(config): Extension<Arc<Config>>,
    Extension(state): Extension<AppState>,
) -> Json<AppResponse> {
    let TracerouteParams { target, version } = params;
    let target = target.trim().to_string();

    if let Err(msg) = validate_target(&target) {
        return Json(AppResponse::Error(msg));
    }

    if let Some(node_config) = config.nodes.iter().find(|n| n.name == node_name) {
        let endpoint = match version.as_str() {
            "4" => "traceroute4",
            "6" => "traceroute6",
            _ => "traceroute",
        };
        let endpoint_with_query = format!("/{}?target={}", endpoint, target);
        match get_text(&state.http_client, node_config, &endpoint_with_query).await {
            Ok(text) => Json(AppResponse::TracerouteResult {
                node: node_name,
                result: text,
            }),
            Err(err_msg) => {
                warn!(
                    node = %node_name,
                    target = %target,
                    error = %err_msg,
                    "Failed to fetch traceroute information"
                );
                Json(AppResponse::Error(err_msg))
            }
        }
    } else {
        Json(AppResponse::Error("Node not found".into()))
    }
}
