use std::{net::IpAddr, sync::Arc};

use axum::extract::{Extension, Path, Query};
use ipnet::IpNet;
use serde::Deserialize;
use tracing::warn;

use crate::{
    config::Config,
    services::request::post_text,
    state::{AppState, AppResponse},
};
use axum::Json;

#[derive(Deserialize)]
pub struct RouteParams {
    target: String,
    #[serde(default)]
    all: bool,
}

pub async fn get_route(
    Path(node_name): Path<String>,
    Query(params): Query<RouteParams>,
    Extension(config): Extension<Arc<Config>>,
    Extension(state): Extension<AppState>,
) -> Json<AppResponse> {
    if let Some(node) = config.nodes.iter().find(|n| n.name == node_name) {
        let target = params.target.trim();
        let is_valid_target = target.parse::<IpAddr>().is_ok() || target.parse::<IpNet>().is_ok();

        if !is_valid_target {
            return Json(AppResponse::Error(
                "Invalid target format (must be IP or CIDR)".into(),
            ));
        }

        let command = if params.all {
            format!("show route for {} all", params.target)
        } else {
            format!("show route for {}", params.target)
        };

        match post_text(&state.http_client, node, "/bird", &command).await {
            Ok(text) => Json(AppResponse::RouteLookupResult {
                node: node_name,
                result: text,
            }),
            Err(err_msg) => {
                warn!(
                    node = %node_name,
                    target = %params.target,
                    error = %err_msg,
                    "Failed to fetch route information"
                );
                Json(AppResponse::Error(err_msg))
            }
        }
    } else {
        Json(AppResponse::Error("Node not found".into()))
    }
}
