use std::str::FromStr;

use chrono::Utc;
use solana_client::rpc_client::GetConfirmedSignaturesForAddress2Config;
use solana_sdk::{pubkey::Pubkey, signature::Signature};
use solana_transaction_status::UiTransactionEncoding;
use tokio::sync::mpsc;
use tracing::{Level, event, instrument};

use crate::{
    AppState,
    db::{
        accounts::{check_account_exists, insert_account},
        transactions::{insert_transactions, insert_transactions_signatures},
    },
    error::AppError,
    message::SyncStatus,
    models::{Account, Transaction, TransactionSignature},
};

#[instrument(skip(sender, error))]
pub async fn send_error_message(sender: mpsc::Sender<SyncStatus>, error: AppError) {
    event!(Level::ERROR, "Error occurred: {error}");

    // Send the error message to the client
    if let Err(err) = sender
        .send(SyncStatus::Error {
            message: error.to_string(),
        })
        .await
    {
        event!(
            Level::ERROR,
            ?err,
            "-> Error occcured may be receiver dropped"
        );
    }
}

async fn send_sync_message(sender: &mpsc::Sender<SyncStatus>, status: SyncStatus) {
    if let Err(err) = sender.send(status).await {
        event!(
            Level::ERROR,
            "Error occcured may be receiver dropped: {}",
            err.to_string()
        );
    }
}

#[instrument(skip(state, sender))]
pub async fn indexer(
    state: AppState,
    sender: mpsc::Sender<SyncStatus>,
    address: String,
) -> Result<(), AppError> {
    // Convert the address str to Address struct instance of Solana account
    let public_key = Pubkey::from_str(&address)?;

    // Before indexing the account, check if it is already indexed
    if check_account_exists(&state.db, &address).await {
        return Err(AppError::BadRequestError(
            "Account is already indexed".to_string(),
        ));
    }

    event!(Level::INFO, "Begin indexing the address");
    send_sync_message(&sender, SyncStatus::Started).await;

    // Get the Solana account data of the address
    let account = state.rpc.get_account(&public_key).await?;
    event!(Level::INFO, ?account);

    let account = Account {
        address: address.clone(),
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

    // Send the account data to the channel
    send_sync_message(&sender, SyncStatus::AccountData { data: account }).await;

    // Get only the latest 20 transaction signatures
    let signatures = state
        .rpc
        .get_signatures_for_address_with_config(
            &public_key,
            GetConfirmedSignaturesForAddress2Config {
                before: None,
                until: None,
                limit: Some(20),
                commitment: None,
            },
        )
        .await?;

    if signatures.is_empty() {
        return Err(AppError::SolanaError(
            "No transactions found for this address".to_string(),
        ));
    }

    let mut txn_signs: Vec<TransactionSignature> = vec![];

    // // Parse the actual transaction signatures to DB format
    for sign in &signatures {
        txn_signs.push(TransactionSignature {
            signature: sign.signature.clone(),
            account_address: address.clone(),
            slot: sign.slot as i64,
            block_time: sign.block_time,
            confirmation_status: serde_json::from_str(&serde_json::to_string(
                &sign.confirmation_status,
            )?)?,
            indexed_at: Utc::now().into(),
        });
    }

    // Insert the transaction signatures into DB
    insert_transactions_signatures(&state.db, &txn_signs).await?;

    // Send the transaction signatures data status to the channel
    send_sync_message(
        &sender,
        SyncStatus::TransactionSignatures {
            fetched: txn_signs.len() as u64,
        },
    )
    .await;

    let mut txns: Vec<Transaction> = vec![];
    for sign in &signatures {
        let signature = Signature::from_str(&sign.signature)?;

        let txn = state
            .rpc
            .get_transaction(&signature, UiTransactionEncoding::JsonParsed)
            .await?;

        txns.push(Transaction {
            signature: sign.signature.clone(),
            account_address: address.clone(),
            slot: txn.slot as i64,
            block_time: txn.block_time,
            transaction: serde_json::to_value(txn.transaction)?,
            indexed_at: Utc::now().into(),
        });
    }

    // Insert the transactions into DB
    insert_transactions(&state.db, &txns).await?;

    // Send the transactions data status to the channel
    send_sync_message(
        &sender,
        SyncStatus::TransactionDetails {
            fetched: txns.len() as u64,
        },
    )
    .await;

    event!(
        Level::INFO,
        "Batch-1 of signatures {} & transactions {} completed",
        signatures.len(),
        txns.len()
    );
    let next_signature = signatures.last().unwrap().signature.clone();
    continue_indexing(
        state,
        sender,
        address.clone(),
        public_key,
        next_signature,
        signatures.len(),
        txns.len(),
    )
    .await?;

    Ok(())
}

#[instrument(skip_all)]
async fn continue_indexing(
    state: AppState,
    sender: mpsc::Sender<SyncStatus>,
    address: String,
    public_key: Pubkey,
    next_signature: String,
    fetched_signatures: usize,
    fetched_transactions: usize,
) -> Result<(), AppError> {
    event!(Level::INFO, "Fetching the rest of the transactions");

    let mut total_signs = fetched_signatures;
    let mut total_txns = fetched_transactions;
    let mut batch = 1;
    const BATCH_SIZE: usize = 10;

    let mut before_signature = next_signature;

    loop {
        // Get the next batch transaction signatures
        let signatures = state
            .rpc
            .get_signatures_for_address_with_config(
                &public_key,
                GetConfirmedSignaturesForAddress2Config {
                    before: Some(Signature::from_str(&before_signature)?),
                    until: None,
                    limit: Some(BATCH_SIZE),
                    commitment: None,
                },
            )
            .await?;

        if signatures.is_empty() {
            event!(Level::INFO, "No more transactions found");
            break;
        }

        let mut txn_signs: Vec<TransactionSignature> = vec![];

        // Parse the transaction signatures to DB format
        for sign in &signatures {
            txn_signs.push(TransactionSignature {
                signature: sign.signature.clone(),
                account_address: address.clone(),
                slot: sign.slot as i64,
                block_time: sign.block_time,
                confirmation_status: serde_json::from_str(&serde_json::to_string(
                    &sign.confirmation_status,
                )?)?,
                indexed_at: Utc::now().into(),
            });
        }

        // Insert the transaction signatures into DB
        insert_transactions_signatures(&state.db, &txn_signs).await?;

        total_signs += signatures.len();

        // Send the transaction signatures data status to the channel
        send_sync_message(
            &sender,
            SyncStatus::TransactionSignatures {
                fetched: total_signs as u64,
            },
        )
        .await;

        let mut txns: Vec<Transaction> = vec![];
        for sign in &signatures {
            let signature = Signature::from_str(&sign.signature)?;

            // Get the transaction details based on the signature
            let txn = state
                .rpc
                .get_transaction(&signature, UiTransactionEncoding::JsonParsed)
                .await?;

            txns.push(Transaction {
                signature: sign.signature.clone(),
                account_address: address.clone(),
                slot: txn.slot as i64,
                block_time: txn.block_time,
                transaction: serde_json::to_value(txn.transaction)?,
                indexed_at: Utc::now().into(),
            });
        }

        // Insert the transactions into DB
        insert_transactions(&state.db, &txns).await?;

        total_txns += txns.len();

        // Send the transaction signatures data status to the channel
        send_sync_message(
            &sender,
            SyncStatus::TransactionDetails {
                fetched: total_txns as u64,
            },
        )
        .await;

        batch += 1;
        event!(
            Level::INFO,
            "Batch-{} of signatures {} & transactions {} completed",
            batch,
            signatures.len(),
            txns.len()
        );

        before_signature = signatures.last().unwrap().signature.clone();

        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    }

    event!(Level::INFO, "Indexing is completed");

    // Send the completed indexing message to the channel
    send_sync_message(&sender, SyncStatus::Completed).await;

    Ok(())
}
