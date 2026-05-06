use common::api::{AppRequest, AppResponse};
use futures::{SinkExt, StreamExt, channel::mpsc, future::Either};
use gloo_net::websocket::{Message, futures::WebSocket};
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;

use crate::{
    store::{AppEvent, LgStateHandle},
    utils::sleep_ms,
};

pub struct WebSocketService;

impl WebSocketService {
    pub fn connect(backend_url: String, state: LgStateHandle) {
        spawn_local(async move {
            let ws_url = backend_url
                .trim_end_matches('/')
                .replace("http://", "ws://")
                .replace("https://", "wss://")
                + "/api/ws";
            let mut ws_failed_count = 0;
            const MAX_WS_FAILURES: u32 = 3;

            loop {
                if ws_failed_count >= MAX_WS_FAILURES {
                    break;
                }

                state.dispatch(AppEvent::SetWsConnecting);

                match WebSocket::open(&ws_url) {
                    Ok(ws) => {
                        let (tx, rx) = mpsc::channel::<AppRequest>(100);
                        let callback = Callback::from(move |req: AppRequest| {
                            let mut tx = tx.clone();
                            spawn_local(async move {
                                let _ = tx.send(req).await;
                            });
                        });

                        state.dispatch(AppEvent::SetWsConnected(callback));

                        let (mut write, read) = ws.split();
                        let mut combined =
                            futures::stream::select(read.map(Either::Left), rx.map(Either::Right));

                        while let Some(item) = combined.next().await {
                            match item {
                                Either::Left(msg) => match msg {
                                    Ok(Message::Text(text)) => {
                                        Self::handle_message(&text, &state);
                                        ws_failed_count = 0;
                                    }
                                    Ok(Message::Bytes(_)) => unreachable!(),
                                    Err(e) => {
                                        tracing::warn!("WebSocket error: {:?}", e);
                                        ws_failed_count += 1;
                                        let _ = write.close().await;
                                        break;
                                    }
                                },
                                Either::Right(req) => {
                                    if let Ok(json) = serde_json::to_string(&req)
                                        && write.send(Message::Text(json)).await.is_err()
                                    {
                                        ws_failed_count += 1;
                                        let _ = write.close().await;
                                        break;
                                    }
                                }
                            }
                        }

                        if ws_failed_count >= MAX_WS_FAILURES {
                            state.dispatch(AppEvent::SetError(
                                "Websocket connection failed".to_string(),
                            ));
                            tracing::error!(
                                "WebSocket failed 3 times. App should switch to HTTP polling.",
                            );
                            state.dispatch(AppEvent::SetWsPollingFallback);
                            break;
                        }

                        state.dispatch(AppEvent::SetWsDisconnected);
                    }
                    Err(_) => {
                        ws_failed_count += 1;
                        if ws_failed_count >= MAX_WS_FAILURES {
                            state.dispatch(AppEvent::SetError(
                                "Websocket connection failed".to_string(),
                            ));
                            tracing::error!(
                                "WebSocket failed 3 times. App should switch to HTTP polling.",
                            );
                            state.dispatch(AppEvent::SetWsPollingFallback);
                        } else {
                            state.dispatch(AppEvent::SetWsDisconnected);
                        }
                    }
                }

                sleep_ms(5000).await;
            }
        });
    }

    fn handle_message(text: &str, state: &LgStateHandle) {
        if let Ok(response) = serde_json::from_str::<AppResponse>(text) {
            crate::services::gateway::ApiGateway::dispatch_response(state, response);
        } else {
            tracing::error!("Unexpected message from the backend: {}", text);
        }
    }
}
