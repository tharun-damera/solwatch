use std::convert::Infallible;
use std::sync::atomic::Ordering;

use axum::{
    extract::{Json, Path, Query, State},
    response::{
        IntoResponse,
        sse::{Event, KeepAlive, Sse},
    },
};
use serde::Deserialize;
use tokio_stream::wrappers::BroadcastStream;
use tokio_stream::{Stream, StreamExt};
use tracing::{error, info, instrument, warn};

use crate::{
    app_state::AppState,
    db::{
        accounts::{get_account, get_address_indexing_state, get_indexer_stats},
        transactions::{get_transaction, get_transaction_signatures, get_transactions},
    },
    error::AppError,
    message::SyncStatus,
    solana,
};

// Entry point API of the app that checks whether the Solana account is indexed or not
#[instrument(skip(state))]
pub async fn account_status(
    Path(address): Path<String>,
    State(state): State<AppState>,
) -> Result<impl IntoResponse, AppError> {
    let state = state.clone();

    let address_state = get_address_indexing_state(&state.db, &address).await?;
    Ok(Json(address_state))
}

fn sync_message_to_event(msg: SyncStatus) -> Event {
    match msg {
        SyncStatus::Indexing => Event::default().event("indexing").data("started"),
        SyncStatus::Syncing => Event::default().event("syncing").data("started"),
        SyncStatus::AccountData(data) => Event::default().event("account-data").data(data),
        SyncStatus::TransactionSignatures(data) => {
            Event::default().event("signatures-fetched").data(data)
        }
        SyncStatus::TransactionDetails(data) => {
            Event::default().event("transactions-fetched").data(data)
        }
        SyncStatus::Error(message) => Event::default().event("error").data(message),
        SyncStatus::Completed => Event::default().event("close").data("close the connection"),
    }
}

// Indexer SSE API is called when the account is not found in DB (not indexed)
// it is used to fetch the account and transaction data via RPC and insert them in DB
// Using broadcast channel to send the sync status messages to all the receivers or the sse clients
pub async fn indexer_sse(
    State(state): State<AppState>,
    Path(address): Path<String>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let session = state.get_or_create_session(&address);
    let receiver = session.sender.subscribe();
    warn!(
        "started AtomicBool value: {}",
        session.started.load(Ordering::Relaxed)
    );

    if !session.started.swap(true, Ordering::AcqRel) {
        let session = session.clone();
        tokio::spawn(async move {
            if let Err(e) = solana::indexer(state.clone(), session.clone(), address.clone()).await {
                session.emit_event(SyncStatus::Error(e.to_string())).await;
                error!(
                    "Error occcured while sending event to channel: {}",
                    e.to_string()
                );
            }
            let removed = state.remove_session(&address);
            info!("Session removed: {}", removed);
        });
    }

    // Convert the past_events iterator to a stream
    // for making sure to send all the events to the late subscribers
    // in case they missed the live events
    let replay_stream = tokio_stream::iter({
        let events = session.past_events.read().await;
        events.clone()
    })
    .map(|event| Ok(sync_message_to_event(event)));

    // Stream the live events as usual
    let live_stream = BroadcastStream::new(receiver).map(|msg_result| match msg_result {
        Ok(msg) => Ok(sync_message_to_event(msg)),
        Err(e) => {
            error!("Broadcast Error: {:#?}", e);
            Ok(Event::default()
                .event("warning")
                .data("client request delayed/lagged"))
        }
    });

    // Combine or Chain the two streams: replay_stream and live_stream
    let stream = replay_stream.chain(live_stream);
    Sse::new(stream).keep_alive(KeepAlive::default())
}

// Refresh SSE API is called to get the latest account and transaction data.
// Basically when all data related to the account in DB is stale and no longer fresh
// and needs to match the on-chain data we call this API
// Using broadcast channel to send the sync status messages to all the receivers or the sse clients
pub async fn refresh_sse(
    State(state): State<AppState>,
    Path(address): Path<String>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let session = state.get_or_create_session(&address);
    let receiver = session.sender.subscribe();
    warn!(
        "started AtomicBool value: {}",
        session.started.load(Ordering::Relaxed)
    );

    if !session.started.swap(true, Ordering::AcqRel) {
        let session = session.clone();
        tokio::spawn(async move {
            if let Err(e) = solana::refresher(state.clone(), session.clone(), address.clone()).await
            {
                session.emit_event(SyncStatus::Error(e.to_string())).await;
                error!(
                    "Error occcured while sending event to channel: {}",
                    e.to_string()
                );
            }

            let removed = state.remove_session(&address);
            info!("Session removed: {}", removed);
        });
    }

    // Convert the past_events iterator to a stream
    // for making sure to send all the events to the late subscribers
    // in case they missed the live events
    let replay_stream = tokio_stream::iter({
        let events = session.past_events.read().await;
        events.clone()
    })
    .map(|event| Ok(sync_message_to_event(event)));

    // Stream the live events as usual
    let live_stream = BroadcastStream::new(receiver).map(|msg_result| match msg_result {
        Ok(msg) => Ok(sync_message_to_event(msg)),
        Err(e) => {
            error!("Broadcast Error: {:#?}", e);
            Ok(Event::default()
                .event("warning")
                .data("client request delayed/lagged"))
        }
    });

    // Combine or Chain the two streams: replay_stream and live_stream
    let stream = replay_stream.chain(live_stream);
    Sse::new(stream).keep_alive(KeepAlive::default())
}

#[instrument(skip(state))]
pub async fn account_data(
    State(state): State<AppState>,
    Path(address): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    let state = state.clone();

    if let Some(account) = get_account(&state.db, &address).await? {
        info!(?account);
        Ok(Json(account))
    } else {
        Err(AppError::NotFound("Account Not Found".to_string()))
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
    Ok(Json(txns))
}

#[instrument(skip(state))]
pub async fn transaction_from_signature(
    State(state): State<AppState>,
    Path((address, signature)): Path<(String, String)>,
) -> Result<impl IntoResponse, AppError> {
    let state = state.clone();

    if let Some(txn) = get_transaction(&state.db, address, signature).await? {
        Ok(Json(txn))
    } else {
        Err(AppError::NotFound("Transaction Not Found".to_string()))
    }
}

#[instrument(skip(state))]
pub async fn indexer_stats(
    State(state): State<AppState>,
    Path(address): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    let state = state.clone();

    let stats = get_indexer_stats(&state.db, &address).await?;
    Ok(Json(stats))
}
