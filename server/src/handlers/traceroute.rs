use std::{convert::Infallible, sync::Arc};

use axum::{
    extract::{Extension, Path, Query},
    response::sse::{Event, Sse},
};
use common::traceroute::TracerouteParams;

use super::response_stream_to_sse;
use crate::{config::Config, services::api::perform_traceroute, state::AppState};

pub async fn proxy_traceroute(
    Query(params): Query<TracerouteParams>,
    Path(node_name): Path<String>,
    Extension(config): Extension<Arc<Config>>,
    Extension(state): Extension<AppState>,
) -> Sse<impl futures_util::Stream<Item = Result<Event, Infallible>>> {
    let TracerouteParams {
        request_id,
        target,
        version,
    } = params;
    let response_stream =
        perform_traceroute(state, config, request_id, node_name, target, Some(version)).await;

    Sse::new(response_stream_to_sse(response_stream))
}
