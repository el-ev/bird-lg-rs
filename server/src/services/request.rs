use crate::config::NodeConfig;
use axum::body::Bytes;
use futures_util::StreamExt;
use reqwest::{Body, Client, RequestBuilder};
use std::io;
use tracing::info;

pub fn build_get(client: &Client, node: &NodeConfig, endpoint: impl AsRef<str>) -> RequestBuilder {
    let url = format!("{}{}", node.url.trim_end_matches('/'), endpoint.as_ref());
    let builder = client.get(&url);
    if let Some(secret) = &node.shared_secret {
        builder.header("x-shared-secret", secret)
    } else {
        builder
    }
}

pub fn build_post<B: Into<Body>>(
    client: &Client,
    node: &NodeConfig,
    endpoint: impl AsRef<str>,
    body: B,
) -> RequestBuilder {
    let url = format!("{}{}", node.url.trim_end_matches('/'), endpoint.as_ref());
    let builder = client.post(&url).body(body);
    if let Some(secret) = &node.shared_secret {
        builder.header("x-shared-secret", secret)
    } else {
        builder
    }
}

pub async fn fetch_stream(
    request: RequestBuilder,
) -> Result<impl futures_util::Stream<Item = Result<Bytes, io::Error>> + 'static, String> {
    match request.send().await {
        Ok(resp) => {
            let status = resp.status();
            if status.is_success() {
                let stream = resp
                    .bytes_stream()
                    .map(|chunk| chunk.map_err(io::Error::other));
                Ok(stream)
            } else {
                let body_text = resp.text().await.unwrap_or_else(|_| "empty".to_string());
                Err(format!("Node returned error: {}", body_text))
            }
        }
        Err(_) => Err("Node is not reachable".to_string()),
    }
}

pub async fn post_stream<T: AsRef<str>>(
    client: &Client,
    node: &NodeConfig,
    url: T,
    command: &str,
) -> Result<impl futures_util::Stream<Item = Result<Bytes, io::Error>> + 'static, String> {
    info!(node = %node.name, url = %url.as_ref(), command = %command, "POST");
    let req = build_post(client, node, url, command.to_string());
    fetch_stream(req).await
}

pub async fn get_stream<T: AsRef<str>>(
    client: &Client,
    node: &NodeConfig,
    url: T,
) -> Result<impl futures_util::Stream<Item = Result<Bytes, io::Error>> + 'static, String> {
    info!(node = %node.name, url = %url.as_ref(), "GET");
    let req = build_get(client, node, url);
    fetch_stream(req).await
}
