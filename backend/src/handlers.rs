use axum::{
    extract::{
        Json, Path, State,
        ws::{WebSocket, WebSocketUpgrade},
    },
    response::IntoResponse,
};
use serde::Serialize;
use sqlx::PgPool;
use tracing::{Level, event, instrument};

use crate::db::accounts::check_account_exists;
use crate::solana;

pub async fn websocket_handler(
    ws: WebSocketUpgrade,
    State(pool): State<PgPool>,
    Path(address): Path<String>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(socket, pool, address))
}

// Websocket handler that handles the Indexing of the Solana account based on the address
async fn handle_socket(mut socket: WebSocket, pool: PgPool, address: String) {
    if let Err(e) = solana::index_address(&mut socket, &pool, &address).await {
        solana::send_error_message(&mut socket, &address, e).await;
    }
}

#[derive(Serialize)]
struct AccountStatus {
    indexed: bool,
}

// Entry point API of the app that checks whether the Solana account is indexed or not
#[instrument(skip(pool))]
pub async fn get_account_status(
    Path(address): Path<String>,
    State(pool): State<PgPool>,
) -> impl IntoResponse {
    event!(Level::INFO, "Checking account indexer status: {address}");

    let indexed = check_account_exists(&pool, address).await;
    Json(AccountStatus { indexed })
}
