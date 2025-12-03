use std::{convert::Infallible, sync::Arc};

use axum::{
    extract::{Extension, Path},
    response::sse::{Event, Sse},
};
use futures_util::stream::StreamExt;

use crate::{config::Config, state::AppState};

pub async fn get_protocol_details(
    Path((node_name, protocol)): Path<(String, String)>,
    Extension(config): Extension<Arc<Config>>,
    Extension(state): Extension<AppState>,
) -> Sse<impl futures_util::Stream<Item = Result<Event, Infallible>>> {
    let response_stream =
        crate::services::api::get_protocol_details(state, config, node_name, protocol).await;

    let sse_stream = response_stream.map(|resp| match serde_json::to_string(&resp) {
        Ok(json) => Ok(Event::default().data(json)),
        Err(_) => Ok(Event::default().data("{\"t\":\"e\",\"error\":\"Serialization failed\"}")),
    });

    Sse::new(sse_stream)
}
