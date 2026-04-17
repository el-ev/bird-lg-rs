use std::{convert::Infallible, sync::Arc};

use axum::{
    extract::{Extension, Path},
    response::sse::{Event, Sse},
};

use super::response_stream_to_sse;
use crate::{config::Config, state::AppState};

pub async fn get_protocol_details(
    Path((node_name, protocol)): Path<(String, String)>,
    Extension(config): Extension<Arc<Config>>,
    Extension(state): Extension<AppState>,
) -> Sse<impl futures_util::Stream<Item = Result<Event, Infallible>>> {
    let response_stream =
        crate::services::api::get_protocol_details(state, config, node_name, protocol).await;

    Sse::new(response_stream_to_sse(response_stream))
}
