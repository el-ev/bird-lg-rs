use std::sync::Arc;

use axum::{Json, extract::Extension};
use common::api::AppResponse;
use futures_util::StreamExt;

use crate::{config::Config, services::api::get_wireguard, state::AppState};

#[utoipa::path(
    get,
    path = "/api/wireguard",
    tag = "network",
    responses(
        (
            status = 200,
            description = "Latest WireGuard snapshot as AppResponse::WireGuard or AppResponse::Error",
            body = common::api::AppResponse
        )
    )
)]
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
