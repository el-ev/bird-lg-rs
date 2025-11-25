use axum::{Json, extract::Extension};
use std::sync::Arc;

use crate::config::{Config, PeeringInfo};

pub async fn get_peering_info(
    Extension(config): Extension<Arc<Config>>,
) -> Json<Option<PeeringInfo>> {
    Json(config.peering.clone())
}
