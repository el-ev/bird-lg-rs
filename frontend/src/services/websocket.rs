use futures::{SinkExt, StreamExt, channel::mpsc, future::Either};
use reqwasm::websocket::{Message, futures::WebSocket};
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;

use crate::store::traceroute::TracerouteAction;
use crate::store::{Action, AppState, NodeTracerouteResult};
use crate::utils::{fetch_json, log_error, parse_traceroute_line, sleep_ms};
use common::models::{NodeStatus, TracerouteHop, WsRequest, WsResponse};

pub struct WebSocketService;

impl WebSocketService {
    pub fn connect(backend_url: String, state: UseReducerHandle<AppState>) {
        spawn_local(async move {
            let ws_url = backend_url
                .trim_end_matches('/')
                .replace("http://", "ws://")
                .replace("https://", "wss://")
                + "/api/ws";
            let mut ws_failed_count = 0;
            const MAX_WS_FAILURES: u32 = 3;

            loop {
                let (tx, rx) = mpsc::channel::<WsRequest>(100);

                let callback = Callback::from(move |req: WsRequest| {
                    let mut tx = tx.clone();
                    spawn_local(async move {
                        let _ = tx.send(req).await;
                    });
                });
                state.dispatch(Action::SetWsSender(callback));

                if ws_failed_count < MAX_WS_FAILURES {
                    match WebSocket::open(&ws_url) {
                        Ok(ws) => {
                            let (mut write, read) = ws.split();
                            let mut combined = futures::stream::select(
                                read.map(Either::Left),
                                rx.map(Either::Right),
                            );

                            while let Some(item) = combined.next().await {
                                match item {
                                    Either::Left(msg) => match msg {
                                        Ok(Message::Text(text)) => {
                                            Self::handle_message(&text, &state);
                                        }
                                        Ok(Message::Bytes(_)) => {}
                                        Err(_) => {
                                            ws_failed_count += 1;
                                            break;
                                        }
                                    },
                                    Either::Right(req) => {
                                        if let Ok(json) = serde_json::to_string(&req)
                                            && write.send(Message::Text(json)).await.is_err()
                                        {
                                            break;
                                        }
                                    }
                                }
                            }
                        }
                        Err(_) => {
                            ws_failed_count += 1;
                            if ws_failed_count >= MAX_WS_FAILURES {
                                state.dispatch(Action::SetError(
                                    "Websocket connection failed".to_string(),
                                ));
                                log_error("WebSocket failed 3 times, falling back to HTTP polling");
                            }
                        }
                    }
                } else {
                    Self::poll_http(&backend_url, &state).await;
                }

                sleep_ms(5000).await;
            }
        });
    }

    fn handle_message(text: &str, state: &UseReducerHandle<AppState>) {
        if let Ok(response) = serde_json::from_str::<WsResponse>(text) {
            match response {
                WsResponse::Protocols(nodes) => {
                    state.dispatch(Action::SetDataReady(true));
                    state.dispatch(Action::ClearError);
                    state.dispatch(Action::SetNodes(nodes));
                }
                WsResponse::NoChange { last_updated } => {
                    state.dispatch(Action::UpdateTimestamp(last_updated));
                }
                WsResponse::TracerouteResult { node, result } => {
                    let hops: Vec<TracerouteHop> = result
                        .lines()
                        .filter(|line| !line.trim().is_empty())
                        .filter_map(parse_traceroute_line)
                        .collect();

                    state.dispatch(Action::Traceroute(TracerouteAction::UpdateResult(
                        node,
                        NodeTracerouteResult::Hops(hops),
                    )));
                }
                WsResponse::RouteLookupResult { node: _, result } => {
                    state.dispatch(Action::RouteLookupResult(result));
                }
                WsResponse::ProtocolDetailsResult {
                    node: _,
                    protocol: _,
                    details,
                } => {
                    let filtered = common::utils::filter_protocol_details(&details);
                    state.dispatch(Action::ProtocolDetailsResult(filtered));
                }
                WsResponse::Error(e) => {
                    log_error(&format!("WS Error: {}", e));
                }
            }
        } else if let Ok(nodes) = serde_json::from_str::<Vec<NodeStatus>>(text) {
            // Legacy/Fallback
            state.dispatch(Action::SetDataReady(true));
            state.dispatch(Action::ClearError);
            state.dispatch(Action::SetNodes(nodes));
        }
    }

    async fn poll_http(backend_url: &str, state: &UseReducerHandle<AppState>) {
        let url = format!("{}/api/protocols", backend_url.trim_end_matches('/'));

        match fetch_json::<Vec<NodeStatus>>(&url).await {
            Ok(nodes) => {
                state.dispatch(Action::SetDataReady(true));
                state.dispatch(Action::ClearError);
                state.dispatch(Action::SetNodes(nodes));
            }
            Err(e) => {
                log_error(&format!("HTTP polling error: {}", e));
                state.dispatch(Action::SetError("Error connecting to backend".to_string()));
            }
        }
    }
}
