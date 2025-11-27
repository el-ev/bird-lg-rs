use crate::config::NodeConfig;
use reqwest::{Body, Client, RequestBuilder};

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

pub async fn fetch_text(request: RequestBuilder) -> Result<String, String> {
    match request.send().await {
        Ok(resp) => {
            let status = resp.status();
            if status.is_success() {
                resp.text()
                    .await
                    .map_err(|_| "Failed to read response body".to_string())
            } else {
                let body_text = resp.text().await.unwrap_or_else(|_| "empty".to_string());
                Err(format!("Node returned error: {}", body_text))
            }
        }
        Err(_) => Err("Node is not reachable".to_string()),
    }
}

pub async fn post_text<T: AsRef<str>>(
    client: &Client,
    node: &NodeConfig,
    url: T,
    command: &str,
) -> Result<String, String> {
    let req = build_post(client, node, url, command.to_string());
    fetch_text(req).await
}

pub async fn get_text<T: AsRef<str>>(
    client: &Client,
    node: &NodeConfig,
    url: T,
) -> Result<String, String> {
    let req = build_get(client, node, url);
    fetch_text(req).await
}
