use std::convert::Infallible;
use std::sync::Arc;

use axum::extract::{Extension, Path, Query};
use axum::response::sse::{Event, Sse};
use futures_util::stream::StreamExt;
use serde::Deserialize;

use crate::{config::Config, services::api::perform_route_lookup, state::AppState};

#[derive(Deserialize)]
pub struct RouteLookupQuery {
    pub target: String,
    #[serde(default)]
    pub all: bool,
}

pub async fn get_route(
    Path(node_name): Path<String>,
    Query(params): Query<RouteLookupQuery>,
    Extension(config): Extension<Arc<Config>>,
    Extension(state): Extension<AppState>,
) -> Sse<impl futures_util::Stream<Item = Result<Event, Infallible>>> {
    let response_stream =
        perform_route_lookup(state, config, node_name, params.target, params.all).await;

    let sse_stream = response_stream.map(|resp| match serde_json::to_string(&resp) {
        Ok(json) => Ok(Event::default().data(json)),
        Err(_) => Ok(Event::default().data("{\"t\":\"e\",\"error\":\"Serialization failed\"}")),
    });

    Sse::new(sse_stream)
}
