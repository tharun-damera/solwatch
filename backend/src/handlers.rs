use axum::{
    extract::{
        Json, Path,
        ws::{WebSocket, WebSocketUpgrade},
    },
    response::IntoResponse,
};
use serde::Serialize;

pub async fn websocket_handler(ws: WebSocketUpgrade) -> impl IntoResponse {
    ws.on_upgrade(handle_socket)
}

async fn handle_socket(mut socket: WebSocket) {
    while let Some(Ok(message)) = socket.recv().await {
        println!("message: {:?}", message);
    }
}

#[derive(Serialize)]
struct AccountStatus {
    indexed: bool,
}

pub async fn get_account_status(Path(address): Path<String>) -> impl IntoResponse {
    println!("Got address: {address}");

    let indexed = true;
    Json(AccountStatus { indexed })
}
