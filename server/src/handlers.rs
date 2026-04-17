use std::convert::Infallible;

use axum::response::sse::Event;
use common::api::AppResponse;
use futures_util::{Stream, StreamExt};

pub mod info;
pub mod ping;
pub mod protocol;
pub mod route;
pub mod status;
pub mod traceroute;
pub mod wireguard;
pub mod ws;

fn response_stream_to_sse<S>(response_stream: S) -> impl Stream<Item = Result<Event, Infallible>>
where
    S: Stream<Item = AppResponse>,
{
    response_stream.map(|response| match serde_json::to_string(&response) {
        Ok(json) => Ok(Event::default().data(json)),
        Err(_) => Ok(Event::default().data("{\"t\":\"e\",\"error\":\"Serialization failed\"}")),
    })
}
