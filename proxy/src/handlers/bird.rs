use std::sync::Arc;

use axum::{body::Body, extract::Extension, response::IntoResponse};
use tokio::io::AsyncWriteExt;
use tokio_util::codec::Framed;
use tracing::{error, info};

use crate::config::Config;
use crate::services::bird::{BirdDecoder, BirdStream, connect};

pub async fn handler(Extension(config): Extension<Arc<Config>>, body: String) -> impl IntoResponse {
    let mut stream = match connect(&config.bind_socket).await {
        Ok(s) => s,
        Err(e) => {
            error!(error = %e, "Failed to connect to bird socket");
            return Body::from(e.to_string());
        }
    };

    let body = if body.ends_with('\n') {
        body
    } else {
        format!("{}\n", body)
    };
    info!("Proxying bird request: {}", body.trim_end());

    if let Err(e) = stream.write_all(body.as_bytes()).await {
        error!(error = %e, "Failed to write bird request");
        return Body::from(e.to_string());
    }

    Body::from_stream(BirdStream {
        inner: Framed::new(stream, BirdDecoder::default()),
        done: false,
    })
}
