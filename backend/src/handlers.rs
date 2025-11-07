use axum::{
    extract::{
        Json, Path, State,
        ws::{WebSocket, WebSocketUpgrade},
    },
    response::IntoResponse,
};
use serde::Serialize;
use sqlx::PgPool;

use crate::db::accounts::check_account_exists;
use crate::solana;

pub async fn websocket_handler(
    ws: WebSocketUpgrade,
    State(pool): State<PgPool>,
    Path(address): Path<String>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(socket, pool, address))
}

async fn handle_socket(socket: WebSocket, pool: PgPool, address: String) {
    solana::index_address(socket, &pool, address).await.ok();
}

#[derive(Serialize)]
struct AccountStatus {
    indexed: bool,
}

pub async fn get_account_status(
    Path(address): Path<String>,
    State(pool): State<PgPool>,
) -> impl IntoResponse {
    println!("Got address: {address}");

    let indexed = check_account_exists(&pool, address).await;
    Json(AccountStatus { indexed })
}
