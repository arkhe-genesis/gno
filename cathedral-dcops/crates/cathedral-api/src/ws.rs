use axum::{
    extract::{State, ws::{WebSocketUpgrade, WebSocket, Message}},
    response::Response,
};
use std::sync::Arc;
use crate::AppState;
use futures_util::{sink::SinkExt, stream::StreamExt};

pub async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<Arc<AppState>>,
) -> Response {
    ws.on_upgrade(|socket| handle_socket(socket, state))
}

async fn handle_socket(socket: WebSocket, state: Arc<AppState>) {
    let (mut sender, mut receiver) = socket.split();
    let mut rx = state.notifier.subscribe();

    tokio::spawn(async move {
        while let Ok(msg) = rx.recv().await {
            if sender.send(Message::Text(msg.into())).await.is_err() { break; }
        }
    });

    while let Some(Ok(_msg)) = receiver.next().await {
        // processa mensagens do cliente (ex: subscribe a filtros)
    }
}
