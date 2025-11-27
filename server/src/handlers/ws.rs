use std::sync::Arc;

use crate::{
    config::Config,
    state::{AppRequest, AppResponse, AppState},
};
use axum::{
    extract::{
        Extension,
        ws::{Message, WebSocket, WebSocketUpgrade},
    },
    response::IntoResponse,
};

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
    let initial_msg = AppResponse::Protocols { data: nodes };
    match serde_json::to_string(&initial_msg) {
        Ok(json) => match socket.send(Message::Text(json.into())).await {
            Ok(_) => {}
            Err(e) => {
                tracing::error!("Failed to send initial message: {}", e);
            }
        },
        Err(e) => {
            tracing::error!("Failed to serialize initial message: {}", e);
        }
    }

    loop {
        tokio::select! {
            Ok(msg) = rx.recv() => {
                match serde_json::to_string(&msg) {
                    Ok(json) => {
                        if let Err(e) = socket.send(Message::Text(json.into())).await {
                            tracing::error!("Failed to send update to client: {}", e);
                            break;
                        }
                    }
                    Err(e) => tracing::error!("Failed to serialize update: {}", e),
                }
            }
            Some(msg) = socket.recv() => {
                match msg {
                    Ok(Message::Text(text)) => {
                        match serde_json::from_str::<AppRequest>(&text) {
                            Ok(req) => {
                                let response = handle_request(req, &state, &config).await;
                                match serde_json::to_string(&response) {
                                    Ok(json) => {
                                        if let Err(e) = socket.send(Message::Text(json.into())).await {
                                            tracing::error!("Failed to send response to client: {}", e);
                                            break;
                                        }
                                    }
                                    Err(e) => tracing::error!("Failed to serialize response: {}", e),
                                }
                            }
                            Err(e) => tracing::warn!("Failed to parse request: {}", e),
                        }
                    }
                    Ok(Message::Close(_)) => break,
                    Err(e) => {
                        tracing::error!("WebSocket error: {}", e);
                        break;
                    }
                    _ => {}
                }
            }
            else => break,
        }
    }
}

async fn handle_request(req: AppRequest, state: &AppState, config: &Arc<Config>) -> AppResponse {
    match req {
        AppRequest::GetProtocols => {
            let nodes = state.nodes.read().unwrap().clone();
            AppResponse::Protocols { data: nodes }
        }
        AppRequest::Traceroute { node, target } => {
            crate::services::api::perform_traceroute(state, config, &node, &target, None).await
        }
        AppRequest::RouteLookup { node, target, all } => {
            crate::services::api::perform_route_lookup(state, config, &node, &target, all).await
        }
        AppRequest::ProtocolDetails { node, protocol } => {
            crate::services::api::get_protocol_details(state, config, &node, &protocol).await
        }
    }
}
