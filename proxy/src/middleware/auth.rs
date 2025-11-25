use std::sync::Arc;

use axum::{
    body::Body,
    extract::Request,
    response::{IntoResponse, Response},
};
use hyper::HeaderMap;

use crate::config::Config;
use tracing::{error, warn};

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

    let connect_info = req
        .extensions()
        .get::<axum::extract::ConnectInfo<std::net::SocketAddr>>()
        .cloned();

    let client_addr = headers
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
        .and_then(|s| s.parse::<std::net::IpAddr>().ok());

    if let Some(secret) = config.shared_secret.as_ref().filter(|s| !s.is_empty()) {
        let header_ok = headers
            .get("x-shared-secret")
            .and_then(|v| v.to_str().ok())
            .map(|s| s == secret)
            .unwrap_or(false);

        if !header_ok {
            warn!("Rejected request due to invalid shared secret");
            return (axum::http::StatusCode::UNAUTHORIZED, "Unauthorized").into_response();
        }
    }

    if let Some(addr) = client_addr {
        for net in config.allowed_nets.iter() {
            if net.contains(&addr) {
                return next.run(req).await;
            }
        }
    }

    warn!(client_ip = ?client_addr, "Rejected request from unauthorized network");

    (axum::http::StatusCode::FORBIDDEN, "Forbidden").into_response()
}
