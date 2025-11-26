use std::sync::Arc;

use crate::{
    config::Config,
    services::request::{build_get, build_post},
    state::{AppState, WsRequest, WsResponse},
};
use axum::{
    extract::{
        Extension,
        ws::{Message, WebSocket, WebSocketUpgrade},
    },
    response::IntoResponse,
};
use common::validate_target;
use ipnet::IpNet;
use std::net::IpAddr;

pub async fn ws_handler(
    ws: WebSocketUpgrade,
    Extension(state): Extension<AppState>,
    Extension(config): Extension<Arc<Config>>,
) -> impl IntoResponse {
    state.record_request();
    ws.on_upgrade(|socket| handle_socket(socket, state, config))
}

async fn handle_socket(mut socket: WebSocket, state: AppState, config: Arc<Config>) {
    let mut rx = state.tx.subscribe();

    // Send initial state
    let nodes = state.nodes.read().unwrap().clone();
    let initial_msg = WsResponse::Protocols(nodes);
    if let Ok(json) = serde_json::to_string(&initial_msg)
        && socket.send(Message::Text(json.into())).await.is_err()
    {
        return;
    }

    loop {
        tokio::select! {
            Ok(msg) = rx.recv() => {
                if let Ok(json) = serde_json::to_string(&msg)
                    && socket.send(Message::Text(json.into())).await.is_err() {
                        break;
                    }
            }
            Some(Ok(msg)) = socket.recv() => {
                match msg {
                    Message::Text(text) => {
                        if let Ok(req) = serde_json::from_str::<WsRequest>(&text) {
                            let response = handle_request(req, &state, &config).await;
                            if let Ok(json) = serde_json::to_string(&response)
                                && socket.send(Message::Text(json.into())).await.is_err() {
                                    break;
                                }
                        }
                    }
                    Message::Close(_) => break,
                    _ => {}
                }
            }
            else => break,
        }
    }
}

async fn handle_request(req: WsRequest, state: &AppState, config: &Config) -> WsResponse {
    match req {
        WsRequest::GetProtocols => {
            let nodes = state.nodes.read().unwrap().clone();
            WsResponse::Protocols(nodes)
        }
        WsRequest::Traceroute { node, target } => {
            let target = target.trim().to_string();
            if let Err(msg) = validate_target(&target) {
                return WsResponse::Error(msg);
            }

            if let Some(node_config) = config.nodes.iter().find(|n| n.name == node) {
                let endpoint_with_query = format!("/traceroute?target={}", target);
                // FIXME: should be streamed
                let req = build_get(&state.http_client, node_config, &endpoint_with_query);

                match req.send().await {
                    Ok(resp) => {
                        if resp.status().is_success() {
                            match resp.text().await {
                                Ok(text) => WsResponse::TracerouteResult { node, result: text },
                                Err(_) => {
                                    WsResponse::Error("Failed to read traceroute response".into())
                                }
                            }
                        } else {
                            WsResponse::Error(format!("Node returned error: {}", resp.status()))
                        }
                    }
                    Err(e) => WsResponse::Error(format!("Failed to contact node: {}", e)),
                }
            } else {
                WsResponse::Error("Node not found".into())
            }
        }
        WsRequest::RouteLookup { node, target, all } => {
            let target_str = target.trim();
            let is_valid_target =
                target_str.parse::<IpAddr>().is_ok() || target_str.parse::<IpNet>().is_ok();

            if !is_valid_target {
                return WsResponse::Error("Invalid target format (must be IP or CIDR)".into());
            }

            if let Some(node_config) = config.nodes.iter().find(|n| n.name == node) {
                let command = if all {
                    format!("show route for {} all", target_str)
                } else {
                    format!("show route for {}", target_str)
                };

                let req = build_post(&state.http_client, node_config, "/bird", command);

                match req.send().await {
                    Ok(resp) => {
                        if resp.status().is_success() {
                            match resp.text().await {
                                Ok(text) => WsResponse::RouteLookupResult { node, result: text },
                                Err(_) => WsResponse::Error("Failed to read route response".into()),
                            }
                        } else {
                            WsResponse::Error(format!("Node returned error: {}", resp.status()))
                        }
                    }
                    Err(e) => WsResponse::Error(format!("Failed to contact node: {}", e)),
                }
            } else {
                WsResponse::Error("Node not found".into())
            }
        }
        WsRequest::ProtocolDetails { node, protocol } => {
            if let Some(node_config) = config.nodes.iter().find(|n| n.name == node) {
                let command = format!("show protocols all {}", protocol);

                let req = build_post(&state.http_client, node_config, "/bird", command);

                match req.send().await {
                    Ok(resp) => {
                        if resp.status().is_success() {
                            match resp.text().await {
                                Ok(text) => WsResponse::ProtocolDetailsResult {
                                    node,
                                    protocol,
                                    details: text,
                                },
                                Err(_) => {
                                    WsResponse::Error("Failed to read protocol details".into())
                                }
                            }
                        } else {
                            WsResponse::Error(format!("Node returned error: {}", resp.status()))
                        }
                    }
                    Err(e) => WsResponse::Error(format!("Failed to contact node: {}", e)),
                }
            } else {
                WsResponse::Error("Node not found".into())
            }
        }
    }
}
