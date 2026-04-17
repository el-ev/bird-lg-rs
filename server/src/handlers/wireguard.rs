use std::sync::Arc;

use axum::{Json, extract::Extension};
use common::api::AppResponse;
use futures_util::StreamExt;

use crate::{config::Config, services::api::get_wireguard, state::AppState};

pub async fn get_wireguard_snapshot(
    Extension(config): Extension<Arc<Config>>,
    Extension(state): Extension<AppState>,
) -> Json<AppResponse> {
    let mut stream = Box::pin(get_wireguard(state, config).await);
    match stream.next().await {
        Some(response) => Json(response),
        None => Json(AppResponse::Error(
            "WireGuard data not available".to_string(),
        )),
    }
}
