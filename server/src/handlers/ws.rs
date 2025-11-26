use crate::state::AppState;
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
) -> impl IntoResponse {
    ws.on_upgrade(|socket| handle_socket(socket, state))
}

async fn handle_socket(mut socket: WebSocket, state: AppState) {
    let mut rx = state.tx.subscribe();

    let nodes = state.nodes.read().unwrap().clone();
    if let Ok(json) = serde_json::to_string(&nodes)
        && socket.send(Message::Text(json.into())).await.is_err()
    {
        return;
    }

    while let Ok(nodes) = rx.recv().await {
        if let Ok(json) = serde_json::to_string(&nodes)
            && socket.send(Message::Text(json.into())).await.is_err()
        {
            break;
        }
    }
}
