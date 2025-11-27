use std::sync::Arc;

use axum::extract::{Extension, Path};

use crate::{
    config::Config,
    state::{AppResponse, AppState},
};
use axum::Json;

pub async fn get_protocol_details(
    Path((node_name, protocol)): Path<(String, String)>,
    Extension(config): Extension<Arc<Config>>,
    Extension(state): Extension<AppState>,
) -> Json<AppResponse> {
    let response =
        crate::services::api::get_protocol_details(&state, &config, &node_name, &protocol).await;
    Json(response)
}
