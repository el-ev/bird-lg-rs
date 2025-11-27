use std::convert::Infallible;
use std::sync::Arc;

use axum::extract::{Extension, Path, Query};
use axum::response::sse::{Event, Sse};
use common::models::TracerouteParams;
use futures_util::stream::StreamExt;

use crate::{config::Config, services::api::perform_traceroute, state::AppState};

pub async fn proxy_traceroute(
    Query(params): Query<TracerouteParams>,
    Path(node_name): Path<String>,
    Extension(config): Extension<Arc<Config>>,
    Extension(state): Extension<AppState>,
) -> Sse<impl futures_util::Stream<Item = Result<Event, Infallible>>> {
    let TracerouteParams { target, version } = params;
    let response_stream = perform_traceroute(state, config, node_name, target, Some(version)).await;

    let sse_stream = response_stream.map(|resp| match serde_json::to_string(&resp) {
        Ok(json) => Ok(Event::default().data(json)),
        Err(_) => Ok(Event::default().data("{\"t\":\"e\",\"error\":\"Serialization failed\"}")),
    });

    Sse::new(sse_stream)
}
