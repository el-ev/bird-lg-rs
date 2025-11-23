use axum::{Json, extract::Extension};

use crate::state::{AppState, NodeStatus};

pub async fn get_status(Extension(state): Extension<AppState>) -> Json<Vec<NodeStatus>> {
    Json(state.nodes.read().unwrap().clone())
}
