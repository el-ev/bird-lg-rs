use std::sync::Arc;

use axum::extract::{Extension, Path, Query};
use serde::Deserialize;

use crate::{
    config::Config,
    services::api::perform_route_lookup,
    state::{AppResponse, AppState},
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
    let response =
        perform_route_lookup(&state, &config, &node_name, &params.target, params.all).await;
    Json(response)
}
