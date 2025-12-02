use std::sync::atomic::Ordering;
use std::sync::{Arc, atomic::AtomicUsize};

use crate::{
    config::Config,
    state::{AppRequest, AppResponse, AppState},
};
use axum::{
    extract::{
        Extension, WebSocketUpgrade,
        ws::{Message, WebSocket},
    },
    response::IntoResponse,
};
use futures_util::{SinkExt, Stream, StreamExt, stream};

struct ConnectionGuard(Arc<AtomicUsize>);

impl ConnectionGuard {
    pub fn new(counter: Arc<AtomicUsize>) -> Self {
        counter.fetch_add(1, Ordering::Relaxed);
        Self(counter)
    }
}

impl Drop for ConnectionGuard {
    fn drop(&mut self) {
        self.0.fetch_sub(1, Ordering::Relaxed);
    }
}

pub async fn ws_handler(
    ws: WebSocketUpgrade,
    Extension(state): Extension<AppState>,
    Extension(config): Extension<Arc<Config>>,
) -> impl IntoResponse {
    state.record_request();
    ws.on_upgrade(|socket| handle_socket(socket, state, config))
}

async fn handle_socket(socket: WebSocket, state: AppState, config: Arc<Config>) {
    let _conn_guard = ConnectionGuard::new(state.active_connections.clone());

    let (mut sender, mut receiver) = socket.split();
    let mut rx = state.tx.subscribe();

    let nodes = state.nodes.read().unwrap().clone();
    let initial_msg = AppResponse::Protocols { data: nodes };
    if let Ok(json) = serde_json::to_string(&initial_msg) {
        if sender.send(Message::Text(json.into())).await.is_err() {
            tracing::error!("Failed to send initial message");
            return;
        }
    } else {
        tracing::error!("Failed to serialize initial message");
        return;
    }

    let state_for_wg = state.clone();
    let config_for_wg = config.clone();
    tokio::spawn(async move {
        let wg_stream =
            crate::services::api::get_wireguard(state_for_wg.clone(), config_for_wg).await;
        let mut stream = Box::pin(wg_stream);
        if let Some(resp) = stream.next().await {
            let _ = state_for_wg.tx.send(resp);
        }
    });

    let (tx, mut mpsc_rx) = tokio::sync::mpsc::unbounded_channel();

    let mut send_task = tokio::spawn(async move {
        loop {
            tokio::select! {
                Ok(broadcast_msg) = rx.recv() => {
                    if let Ok(json) = serde_json::to_string(&broadcast_msg)
                        && sender.send(Message::Text(json.into())).await.is_err()
                    {
                        tracing::error!("Failed to send broadcast update");
                        break;
                    }
                }
                Some(msg) = mpsc_rx.recv() => {
                    if let Ok(json) = serde_json::to_string(&msg)
                        && sender.send(Message::Text(json.into())).await.is_err()
                    {
                        tracing::error!("Failed to send response");
                        break;
                    }
                }
                else => break,
            }
        }
    });

    let state_clone = state.clone();
    let config_clone = config.clone();
    let mut recv_task = tokio::spawn(async move {
        while let Some(Ok(msg)) = receiver.next().await {
            match msg {
                Message::Text(text) => {
                    if let Ok(req) = serde_json::from_str::<AppRequest>(&text) {
                        let state_c = state_clone.clone();
                        let config_c = config_clone.clone();
                        let tx_c = tx.clone();

                        tokio::spawn(async move {
                            let response_stream = handle_request(req, state_c, config_c).await;
                            let mut stream = Box::pin(response_stream);

                            while let Some(resp) = stream.next().await {
                                if tx_c.send(resp).is_err() {
                                    break;
                                }
                            }
                        });
                    }
                }
                Message::Close(_) => break,
                _ => {}
            }
        }
    });

    tokio::select! {
        _ = (&mut send_task) => recv_task.abort(),
        _ = (&mut recv_task) => send_task.abort(),
    }
}

async fn handle_request(
    req: AppRequest,
    state: AppState,
    config: Arc<Config>,
) -> impl Stream<Item = AppResponse> {
    match req {
        AppRequest::GetProtocols => {
            let nodes = state.nodes.read().unwrap().clone();
            stream::once(async move { AppResponse::Protocols { data: nodes } }).left_stream()
        }
        AppRequest::GetWireGuard => crate::services::api::get_wireguard(state, config)
            .await
            .right_stream(),
        AppRequest::Traceroute {
            node,
            target,
            version,
        } => {
            let version = if version.is_empty() {
                None
            } else {
                Some(version)
            };

            crate::services::api::perform_traceroute(state, config, node, target, version)
                .await
                .right_stream()
        }
        AppRequest::RouteLookup { node, target, all } => {
            crate::services::api::perform_route_lookup(state, config, node, target, all)
                .await
                .right_stream()
        }
        AppRequest::ProtocolDetails { node, protocol } => {
            crate::services::api::get_protocol_details(state, config, node, protocol)
                .await
                .right_stream()
        }
    }
}
