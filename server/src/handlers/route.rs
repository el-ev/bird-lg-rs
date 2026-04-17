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
