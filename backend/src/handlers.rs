use axum::{
    extract::{
        Json, Path, State,
        ws::{WebSocket, WebSocketUpgrade},
    },
    response::IntoResponse,
};
use serde::Serialize;
use tracing::{Level, event, instrument};

use crate::{AppState, db::accounts::check_account_exists, solana};

pub async fn websocket_handler(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
    Path(address): Path<String>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(socket, state.clone(), address))
}

// Websocket handler that handles the Indexing of the Solana account based on the address
async fn handle_socket(mut socket: WebSocket, state: AppState, address: String) {
    if let Err(e) = solana::index_address(&mut socket, state, &address).await {
        solana::send_error_message(&mut socket, &address, e).await;
    }
}

#[derive(Serialize)]
struct AccountStatus {
    indexed: bool,
}

// Entry point API of the app that checks whether the Solana account is indexed or not
#[instrument(skip(state))]
pub async fn get_account_status(
    Path(address): Path<String>,
    State(state): State<AppState>,
) -> impl IntoResponse {
    let state = state.clone();

    event!(Level::INFO, "Checking account indexer status: {address}");
    let indexed = check_account_exists(&state.db, &address).await;

    Json(AccountStatus { indexed })
}
