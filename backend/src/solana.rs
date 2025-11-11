use std::str::FromStr;

use axum::extract::ws::{Message, WebSocket};
use chrono::Utc;
use serde::Serialize;
use solana_client::rpc_client::GetConfirmedSignaturesForAddress2Config;
use solana_sdk::{pubkey::Pubkey, signature::Signature};
use solana_transaction_status::UiTransactionEncoding;
use tracing::{Level, event, instrument};

use crate::{
    AppState,
    db::accounts::{check_account_exists, insert_account, insert_transactions},
    error::AppError,
    message::IndexingMessage,
    models::{Account, TransactionSignature},
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
    if check_account_exists(&state.db, address).await {
        return Err(AppError::BadRequestError(
            "Account is already indexed".to_string(),
        ));
    }

    event!(Level::INFO, "Begin indexing the address");
    send_message(socket, IndexingMessage::Started { address: &address }).await;

    // Get the Solana account data of the address
    let account = state.rpc.get_account(&public_key).await?;
    event!(Level::INFO, ?account);

    let account = Account {
        address: address.to_string(),
        lamports: account.lamports as i64,
        owner: account.owner.to_string(),
        executable: account.executable,
        data_length: account.data.len() as i64,
        rent_epoch: account.rent_epoch as i64,
        indexed_at: Utc::now().into(),
        last_updated_at: Utc::now().into(),
    };

    // Insert the account data into DB
    insert_account(&state.db, &account).await?;

    // Send the account data to the client via socket communication
    send_message(socket, IndexingMessage::AccountData(account)).await;

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

    let mut txn_signs: Vec<TransactionSignature> = vec![];

    // Parse the actual transaction signatures to DB format
    for sign in signatures {
        txn_signs.push(TransactionSignature {
            signature: sign.signature.clone(),
            account_address: address.to_string(),
            slot: sign.slot as i64,
            block_time: sign.block_time,
            confirmation_status: serde_json::from_str(&serde_json::to_string(
                &sign.confirmation_status,
            )?)?,
            indexed_at: Utc::now().into(),
        });

        let signature = Signature::from_str(&sign.signature)?;
        let txn = state
            .rpc
            .get_transaction(&signature, UiTransactionEncoding::JsonParsed)
            .await?;
        event!(Level::INFO, ?txn);
    }

    // Insert the parsed transactions into DB
    insert_transactions(&state.db, &txn_signs).await?;

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
