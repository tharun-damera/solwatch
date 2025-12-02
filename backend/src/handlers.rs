use std::convert::Infallible;

use axum::{
    extract::{Json, Path, Query, State},
    response::{
        IntoResponse,
        sse::{Event, KeepAlive, Sse},
    },
};
use futures::{Stream, StreamExt};
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use tracing::{Level, event, instrument};

use crate::{
    AppState,
    db::{
        accounts::{check_account_exists, get_account},
        transactions::{get_transaction, get_transaction_signatures, get_transactions},
    },
    error::AppError,
    message::SyncStatus,
    solana,
};

#[derive(Serialize)]
struct AccountStatus {
    indexed: bool,
}

// Entry point API of the app that checks whether the Solana account is indexed or not
#[instrument(skip(state))]
pub async fn get_account_status(
    Path(address): Path<String>,
    State(state): State<AppState>,
) -> Result<impl IntoResponse, AppError> {
    let state = state.clone();

    let indexed = check_account_exists(&state.db, &address).await;
    if indexed {
        Ok(Json(AccountStatus { indexed }))
    } else {
        Err(AppError::NotFoundError("Account Not found".to_string()))
    }
}

fn sync_message_to_event(msg: SyncStatus) -> Event {
    match msg {
        SyncStatus::Started => Event::default().event("started"),
        SyncStatus::AccountData { data } => Event::default().event("account-data").data(data),
        SyncStatus::TransactionSignatures { fetched } => Event::default()
            .event("signatures-fetched")
            .data(fetched.to_string()),
        SyncStatus::TransactionDetails { fetched } => Event::default()
            .event("transactions-fetched")
            .data(fetched.to_string()),
        SyncStatus::Error { message } => Event::default().event("error").data(message),
        SyncStatus::Completed => Event::default().event("close"),
    }
}

// Indexer SSE API is called when the account is not found in DB (not indexed)
// it is used to fetch the account and transaction data via RPC and insert them in DB
pub async fn indexer_sse(
    State(state): State<AppState>,
    Path(address): Path<String>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let state = state.clone();

    let (sender, receiver) = mpsc::channel(10);
    tokio::spawn(async move {
        if let Err(e) = solana::indexer(state, sender.clone(), address).await {
            solana::send_error_message(sender, e).await;
        }
    });

    let stream = ReceiverStream::new(receiver).map(|msg| Ok(sync_message_to_event(msg)));

    Sse::new(stream).keep_alive(KeepAlive::default())
}

// Refresh SSE API is called to get the latest account and transaction data.
// Basically when all data related to the account in DB is stale and no longer fresh
// and needs to match the on-chain data we call this API
pub async fn refresh_sse(
    State(state): State<AppState>,
    Path(address): Path<String>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let state = state.clone();

    let (sender, receiver) = mpsc::channel(10);
    tokio::spawn(async {
        if let Err(e) = solana::refresher(state, sender.clone(), address).await {
            solana::send_error_message(sender, e).await;
        }
    });

    let stream = ReceiverStream::new(receiver.into()).map(|msg| Ok(sync_message_to_event(msg)));

    Sse::new(stream).keep_alive(KeepAlive::default())
}

#[instrument(skip(state))]
pub async fn get_account_data(
    State(state): State<AppState>,
    Path(address): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    let state = state.clone();

    if let Some(account) = get_account(&state.db, &address).await? {
        event!(Level::INFO, ?account);
        Ok(Json(account))
    } else {
        Err(AppError::NotFoundError("Account Not Found!".to_string()))
    }
}

#[derive(Debug, Deserialize)]
pub struct Pagination {
    skip: u64,
    limit: i64,
}

#[instrument(skip(state))]
pub async fn transaction_signatures(
    State(state): State<AppState>,
    Path(address): Path<String>,
    Query(pagination): Query<Pagination>,
) -> Result<impl IntoResponse, AppError> {
    let state = state.clone();

    let txns =
        get_transaction_signatures(&state.db, address, pagination.skip, pagination.limit).await?;
    event!(Level::INFO, ?txns);

    Ok(Json(txns))
}

#[instrument(skip(state))]
pub async fn transactions(
    State(state): State<AppState>,
    Path(address): Path<String>,
    Query(pagination): Query<Pagination>,
) -> Result<impl IntoResponse, AppError> {
    let state = state.clone();

    let txns = get_transactions(&state.db, address, pagination.skip, pagination.limit).await?;
    event!(Level::INFO, ?txns);

    Ok(Json(txns))
}

#[instrument(skip(state))]
pub async fn transaction_from_signature(
    State(state): State<AppState>,
    Path((address, signature)): Path<(String, String)>,
) -> Result<impl IntoResponse, AppError> {
    let state = state.clone();

    if let Some(txn) = get_transaction(&state.db, address, signature).await? {
        event!(Level::INFO, ?txn);
        Ok(Json(txn))
    } else {
        Err(AppError::NotFoundError(
            "Transaction Not Found!".to_string(),
        ))
    }
}
