use std::sync::Arc;

use axum::{
    body::Body,
    extract::Request,
    response::{IntoResponse, Response},
};
use hyper::HeaderMap;
use tracing::{error, warn};

use crate::config::Config;

pub async fn auth_middleware(
    headers: HeaderMap,
    req: Request<Body>,
    next: axum::middleware::Next,
) -> Response {
    let config = match req.extensions().get::<Arc<Config>>().cloned() {
        Some(cfg) => cfg,
        None => {
            error!("Request missing proxy config extension");
            return (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                "Server error",
            )
                .into_response();
        }
    };

    let client_addr = extract_client_addr(&headers, &req);

    if !has_valid_shared_secret(&config, &headers) {
        warn!("Rejected request due to invalid shared secret");
        return (axum::http::StatusCode::UNAUTHORIZED, "Unauthorized").into_response();
    }

    if is_allowed_client(&config, client_addr) {
        return next.run(req).await;
    }

    warn!(client_ip = ?client_addr, "Rejected request from unauthorized network");

    (axum::http::StatusCode::FORBIDDEN, "Forbidden").into_response()
}

fn extract_client_addr(headers: &HeaderMap, req: &Request<Body>) -> Option<std::net::IpAddr> {
    let connect_info = req
        .extensions()
        .get::<axum::extract::ConnectInfo<std::net::SocketAddr>>()
        .cloned();

    headers
        .get("x-forwarded-for")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.split(',').next().map(|s| s.trim().to_string()))
        .or_else(|| {
            headers
                .get("x-real-ip")
                .and_then(|v| v.to_str().ok())
                .map(|s| s.trim().to_string())
        })
        .or_else(|| connect_info.map(|info| info.0.ip().to_string()))
        .as_deref()
        .and_then(|s| s.parse::<std::net::IpAddr>().ok())
}

fn has_valid_shared_secret(config: &Config, headers: &HeaderMap) -> bool {
    let Some(secret) = config
        .shared_secret
        .as_ref()
        .filter(|value| !value.is_empty())
    else {
        return true;
    };

    headers
        .get("x-shared-secret")
        .and_then(|v| v.to_str().ok())
        .map(|value| value == secret)
        .unwrap_or(false)
}

fn is_allowed_client(config: &Config, client_addr: Option<std::net::IpAddr>) -> bool {
    let Some(client_addr) = client_addr else {
        return false;
    };

    config
        .allowed_nets
        .iter()
        .any(|net| net.contains(&client_addr))
}
