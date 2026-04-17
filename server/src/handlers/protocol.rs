use std::{convert::Infallible, sync::Arc};

use axum::{
    extract::{Extension, Path, Query},
    response::sse::{Event, Sse},
};
use serde::Deserialize;

use super::response_stream_to_sse;
use crate::{config::Config, state::AppState};

#[derive(Deserialize)]
pub struct ProtocolDetailsQuery {
    pub request_id: String,
}

pub async fn get_protocol_details(
    Path((node_name, protocol)): Path<(String, String)>,
    Query(params): Query<ProtocolDetailsQuery>,
    Extension(config): Extension<Arc<Config>>,
    Extension(state): Extension<AppState>,
) -> Sse<impl futures_util::Stream<Item = Result<Event, Infallible>>> {
    let response_stream = crate::services::api::get_protocol_details(
        state,
        config,
        params.request_id,
        node_name,
        protocol,
    )
    .await;

    Sse::new(response_stream_to_sse(response_stream))
}
