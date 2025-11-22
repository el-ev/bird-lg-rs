use std::sync::Arc;

use axum::{body::Body, extract::Extension, response::IntoResponse};
use tokio::io::AsyncWriteExt;
use tokio_util::codec::Framed;

use crate::config::Config;
use crate::services::bird::{BirdDecoder, BirdStream, connect};

pub async fn handler(Extension(config): Extension<Arc<Config>>, body: String) -> impl IntoResponse {
    let mut stream = match connect(&config.bind_socket).await {
        Ok(s) => s,
        Err(e) => return Body::from(e),
    };

    let body = if body.ends_with('\n') {
        body
    } else {
        format!("{}\n", body)
    };

    if let Err(e) = stream.write_all(body.as_bytes()).await {
        return Body::from(e.to_string());
    }

    Body::from_stream(BirdStream {
        inner: Framed::new(stream, BirdDecoder::default()),
        done: false,
    })
}
