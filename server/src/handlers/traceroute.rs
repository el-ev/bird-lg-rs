use std::{convert::Infallible, sync::Arc};

use axum::{
    extract::{Extension, Path, Query},
    response::sse::{Event, Sse},
};
use common::traceroute::TracerouteParams;

use super::response_stream_to_sse;
use crate::{config::Config, services::api::perform_traceroute, state::AppState};

#[utoipa::path(
    get,
    path = "/api/traceroute/{node_name}",
    tag = "tools",
    params(
        ("node_name" = String, Path, description = "Node name"),
        ("request_id" = String, Query, description = "Client-supplied request identifier"),
        ("target" = String, Query, description = "Traceroute target"),
        ("version" = Option<String>, Query, description = "Optional IP version override")
    ),
    responses(
        (
            status = 200,
            description = "Server-Sent Events stream. Each event data field is a JSON-serialized AppResponse traceroute message.",
            content_type = "text/event-stream",
            body = String
        )
    )
)]
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
