use std::str::FromStr;

use axum::extract::ws::{Message, WebSocket};
use serde::Serialize;
use solana_client::rpc_client::GetConfirmedSignaturesForAddress2Config;
use solana_sdk::pubkey::Pubkey;
use tracing::{Level, event, instrument};

use crate::{
    db::accounts::{check_account_exists, insert_account, insert_transactions},
    error::AppError,
    message::IndexingMessage,
    models::{AccountCreate, TransactionSignatureCreate},
    routes::AppState,
};

pub async fn send_message<T: Serialize>(socket: &mut WebSocket, value: T) {
    // Convert the passed-in value to a json string
    // then to the Message::Text for sending it to the client via socket
    match serde_json::to_string(&value) {
        Ok(msg) => {
            event!(Level::INFO, "Sending a message: {msg}");
            if let Err(e) = socket.send(Message::Text(msg.into())).await {
                event!(Level::ERROR, "Failed to send the message: {e}");
            }
        }
        Err(e) => {
            event!(Level::ERROR, "Failed to seriliaze message: {e}");
        }
    }
}

#[instrument(skip(socket, error))]
pub async fn send_error_message(socket: &mut WebSocket, address: &str, error: AppError) {
    event!(Level::ERROR, "Error occurred: {error}");

    // Send the error message to the client
    send_message(
        socket,
        IndexingMessage::Error {
            address,
            message: &error.to_string(),
        },
    )
    .await;
}

#[instrument(skip(socket, state))]
pub async fn index_address(
    socket: &mut WebSocket,
    state: AppState,
    address: &str,
) -> Result<(), AppError> {
    // Convert the address str to Address struct instance of Solana account
    let public_key = Pubkey::from_str(address)?;

    // Before indexing the account, check if it is already indexed
    if check_account_exists(&state.pool, address.to_string()).await {
        return Err(AppError::BadRequestError(
            "Account is already indexed".to_string(),
        ));
    }

    event!(Level::INFO, "Begin indexing the address");
    send_message(socket, IndexingMessage::Started { address: &address }).await;

    // Get the Solana account data of the address
    let account = state.rpc.get_account(&public_key).await?;
    event!(Level::INFO, ?account);

    // Insert the account data into DB
    let inserted_acc = insert_account(
        &state.pool,
        AccountCreate {
            address: address.to_string(),
            lamports: account.lamports as i64,
            owner: account.owner.to_string(),
            executable: account.executable,
            data_length: account.data.len() as i64,
            rent_epoch: account.rent_epoch as i64,
        },
    )
    .await?;
    event!(Level::INFO, ?inserted_acc);

    // Send the account data to the client via socket communication
    send_message(socket, IndexingMessage::AccountData(inserted_acc)).await;

    // Get only the latest 100 transaction signatures
    let signatures = state
        .rpc
        .get_signatures_for_address_with_config(
            &public_key,
            GetConfirmedSignaturesForAddress2Config {
                before: None,
                until: None,
                limit: Some(100),
                commitment: None,
            },
        )
        .await?;
    event!(Level::INFO, ?signatures);

    let mut txn_signs: Vec<TransactionSignatureCreate> = vec![];

    // Parse the actual transaction signatures to DB format
    for sign in signatures {
        txn_signs.push(TransactionSignatureCreate {
            signature: sign.signature,
            account_address: address.to_string(),
            slot: sign.slot as i64,
            block_time: sign.block_time,
            confirmation_status: serde_json::from_str(&serde_json::to_string(
                &sign.confirmation_status,
            )?)?,
        });
    }

    // Insert the parsed transactions into DB
    let query_result = insert_transactions(&state.pool, &txn_signs).await?;
    event!(
        Level::INFO,
        "Insert transactions query result: {query_result:?}"
    );

    // Send the parsed transactions to the client
    send_message(
        socket,
        IndexingMessage::TransactionSignatures { data: &txn_signs },
    )
    .await;

    send_message(socket, IndexingMessage::Completed { address: &address }).await;
    event!(Level::INFO, "Indexing has ended.");

    Ok(())
}
