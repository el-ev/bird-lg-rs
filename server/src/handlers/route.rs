use std::{convert::Infallible, sync::Arc};

use axum::{
    extract::{Extension, Path, Query},
    response::sse::{Event, Sse},
};
use serde::Deserialize;

use super::response_stream_to_sse;
use crate::{config::Config, services::api::perform_route_lookup, state::AppState};

#[derive(Deserialize)]
pub struct RouteLookupQuery {
    pub request_id: String,
    pub target: String,
    #[serde(default)]
    pub all: bool,
}

#[utoipa::path(
    get,
    path = "/api/routes/{node_name}",
    tag = "tools",
    params(
        ("node_name" = String, Path, description = "Node name"),
        ("request_id" = String, Query, description = "Client-supplied request identifier"),
        ("target" = String, Query, description = "Route or prefix to look up"),
        ("all" = Option<bool>, Query, description = "Return all matching routes when true")
    ),
    responses(
        (
            status = 200,
            description = "Server-Sent Events stream. Each event data field is a JSON-serialized AppResponse route-lookup message.",
            content_type = "text/event-stream",
            body = String
        )
    )
)]
pub async fn get_route(
    Path(node_name): Path<String>,
    Query(params): Query<RouteLookupQuery>,
    Extension(config): Extension<Arc<Config>>,
    Extension(state): Extension<AppState>,
) -> Sse<impl futures_util::Stream<Item = Result<Event, Infallible>>> {
    let response_stream = perform_route_lookup(
        state,
        config,
        params.request_id,
        node_name,
        params.target,
        params.all,
    )
    .await;

    Sse::new(response_stream_to_sse(response_stream))
}
