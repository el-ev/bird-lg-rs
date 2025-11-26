use futures::StreamExt;
use gloo_net::websocket::{Message, futures::WebSocket};
use reqwasm::http::Request;
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;

use crate::models::NodeStatus;
use crate::store::{Action, AppState};
use crate::utils::{log_error, sleep_ms};

pub struct WebSocketService;

impl WebSocketService {
    pub fn connect(backend_url: String, state: UseReducerHandle<AppState>) {
        spawn_local(async move {
            let ws_url = Self::construct_ws_url(&backend_url);

            let mut ws_failed_count = 0;
            const MAX_WS_FAILURES: u32 = 3;

            loop {
                if ws_failed_count < MAX_WS_FAILURES {
                    match WebSocket::open(&ws_url) {
                        Ok(ws) => {
                            let (_, mut read) = ws.split();

                            while let Some(msg) = read.next().await {
                                match msg {
                                    Ok(Message::Text(text)) => {
                                        if let Ok(nodes) =
                                            serde_json::from_str::<Vec<NodeStatus>>(&text)
                                        {
                                            state.dispatch(Action::SetDataReady(true));
                                            state.dispatch(Action::ClearError);
                                            state.dispatch(Action::SetNodes(nodes));
                                        }
                                    }
                                    Err(_) => {
                                        ws_failed_count += 1;
                                        if ws_failed_count >= MAX_WS_FAILURES {
                                            log_error(
                                                "WebSocket failed 3 times, falling back to HTTP polling",
                                            );
                                            state.dispatch(Action::SetError(
                                                "Websocket connection failed".to_string(),
                                            ));
                                            break;
                                        }
                                    }
                                    _ => {}
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

    fn construct_ws_url(backend_url: &str) -> String {
        let url = backend_url.trim_end_matches('/');
        if let Some(rest) = url.strip_prefix("https://") {
            format!("wss://{}/api/ws", rest)
        } else if let Some(rest) = url.strip_prefix("http://") {
            format!("ws://{}/api/ws", rest)
        } else {
            unreachable!()
        }
    }

    async fn poll_http(backend_url: &str, state: &UseReducerHandle<AppState>) {
        let url = format!("{}/api/protocols", backend_url.trim_end_matches('/'));

        match Request::get(&url).send().await {
            Ok(resp) if resp.ok() => {
                if let Ok(nodes) = resp.json::<Vec<NodeStatus>>().await {
                    state.dispatch(Action::SetDataReady(true));
                    state.dispatch(Action::ClearError);
                    state.dispatch(Action::SetNodes(nodes));
                }
            }
            Ok(resp) => {
                log_error(&format!(
                    "HTTP polling failed with status {}",
                    resp.status()
                ));
            }
            Err(e) => {
                log_error(&format!("HTTP polling error: {}", e));
                state.dispatch(Action::SetError("Error connecting to backend".to_string()));
            }
        }
    }
}
