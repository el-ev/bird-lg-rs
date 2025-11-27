use std::sync::Arc;

use axum::extract::{Extension, Path, Query};
use common::models::TracerouteParams;

use crate::{
    config::Config,
    services::api::perform_traceroute,
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
    let response = perform_traceroute(&state, &config, &node_name, &target, Some(&version)).await;
    Json(response)
}
