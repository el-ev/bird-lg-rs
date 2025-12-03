use common::api::{AppRequest, AppResponse};
use futures::{SinkExt, StreamExt, channel::mpsc, future::Either};
use reqwasm::websocket::{Message, futures::WebSocket};
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;

use crate::{
    store::{Action, LgStateHandle},
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
                let (tx, rx) = mpsc::channel::<AppRequest>(100);

                let callback = Callback::from(move |req: AppRequest| {
                    let mut tx = tx.clone();
                    spawn_local(async move {
                        let _ = tx.send(req).await;
                    });
                });
                state.dispatch(Action::SetWsSender(callback));

                if ws_failed_count >= MAX_WS_FAILURES {
                    break;
                }

                match WebSocket::open(&ws_url) {
                    Ok(ws) => {
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
                            tracing::error!(
                                "WebSocket failed 3 times. App should switch to HTTP polling.",
                            );
                            state.dispatch(Action::ClearWsSender);
                        }
                    }
                }

                sleep_ms(5000).await;
            }
        });
    }

    fn handle_message(text: &str, state: &LgStateHandle) {
        if let Ok(response) = serde_json::from_str::<AppResponse>(text) {
            crate::services::response_handler::handle_app_response(response, state);
        } else {
            tracing::error!("Unexpected message from the backend: {}", text);
        }
    }
}
