use std::str::FromStr;

use chrono::Utc;
use mongodb::bson::DateTime as BsonDateTime;
use solana_client::rpc_client::GetConfirmedSignaturesForAddress2Config;
use solana_sdk::{pubkey::Pubkey, signature::Signature};
use solana_transaction_status::UiTransactionEncoding;
use tokio::sync::broadcast;
use tracing::{Level, event, instrument};

use crate::{
    app_state::AppState,
    db::{
        accounts::{
            check_account_exists, insert_account, insert_address_indexing_state, update_account,
            update_address_indexing_state,
        },
        transactions::{
            get_latest_signature, get_signatures_count, get_transactions_count,
            insert_transactions, insert_transactions_signatures,
        },
    },
    error::AppError,
    message::SyncStatus,
    models::{
        Account, AddressIndexingState, IndexingState, Transaction, TransactionSignature,
        UpdateAccount, UpdateAddressIndexingState,
    },
};

fn bson_current_time() -> BsonDateTime {
    BsonDateTime::from_millis(Utc::now().timestamp_millis())
}

async fn tokio_sleep(milliseconds: u64) {
    tokio::time::sleep(std::time::Duration::from_millis(milliseconds)).await;
}

#[derive(Debug, serde::Serialize)]
struct TotalFetch {
    total: u64,
    fetched: u64,
}

#[instrument(skip(sender, error))]
pub async fn send_error_message(sender: broadcast::Sender<SyncStatus>, error: AppError) {
    event!(Level::ERROR, "Error occurred: {error}");

    // Send the error message to the client
    if let Err(err) = sender.send(SyncStatus::Error(error.to_string())) {
        event!(
            Level::ERROR,
            "Error occcured may be receiver dropped: {}",
            err.to_string()
        );
    }
}

async fn send_sync_message(sender: &broadcast::Sender<SyncStatus>, status: SyncStatus) {
    if let Err(err) = sender.send(status) {
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
    sender: broadcast::Sender<SyncStatus>,
    address: String,
) -> Result<(), AppError> {
    // Convert the address str to Address struct instance of Solana account
    let public_key = Pubkey::from_str(&address)?;

    // Before indexing the account, check if it is already indexed
    if check_account_exists(&state.db, &address).await {
        return Err(AppError::BadRequest(
            "Account is already indexed".to_string(),
        ));
    }

    // Insert the indexing state of the address for tracking purposes
    insert_address_indexing_state(
        &state.db,
        AddressIndexingState {
            address: address.clone(),
            state: IndexingState::Indexing,
            created_at: bson_current_time(),
            updated_at: bson_current_time(),
        },
    )
    .await?;

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
        indexed_at: bson_current_time(),
        last_updated_at: bson_current_time(),
    };

    // Insert the account data into DB
    insert_account(&state.db, &account).await?;

    // Send the account data to the channel
    send_sync_message(
        &sender,
        SyncStatus::AccountData(serde_json::to_string(&account)?),
    )
    .await;

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
        return Err(AppError::Solana(
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
            indexed_at: bson_current_time(),
        });
    }

    // Insert the transaction signatures into DB
    insert_transactions_signatures(&state.db, &txn_signs).await?;

    // Get the total transaction signatures count of the account in DB
    let sign_count = get_signatures_count(&state.db, &address).await?;

    // Send the transaction signatures data status to the channel
    send_sync_message(
        &sender,
        SyncStatus::TransactionSignatures(serde_json::to_string(&TotalFetch {
            total: sign_count,
            fetched: txn_signs.len() as u64,
        })?),
    )
    .await;

    let mut txns: Vec<Transaction> = vec![];
    for sign in &signatures {
        let signature = Signature::from_str(&sign.signature)?;

        // Sleep again since these are individual RPC calls to get each transaction
        tokio_sleep(100).await;

        // Fetch individual transaction based on signature
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
            indexed_at: bson_current_time(),
        });
    }

    // Insert the transactions into DB
    insert_transactions(&state.db, &txns).await?;

    // Get the total transactions count of the account in DB
    let txn_count = get_transactions_count(&state.db, &address).await?;

    // Send the transactions data status to the channel
    send_sync_message(
        &sender,
        SyncStatus::TransactionDetails(serde_json::to_string(&TotalFetch {
            total: txn_count,
            fetched: txns.len() as u64,
        })?),
    )
    .await;

    event!(
        Level::INFO,
        "Batch-1 of signatures {} & transactions {} completed",
        signatures.len(),
        txns.len()
    );
    let next_signature = signatures.last().unwrap().signature.clone();
    continue_sync(
        state,
        sender,
        address.clone(),
        public_key,
        Some(Signature::from_str(&next_signature)?),
        None,
        signatures.len(),
        txns.len(),
    )
    .await?;

    Ok(())
}

#[instrument(skip_all)]
async fn continue_sync(
    state: AppState,
    sender: broadcast::Sender<SyncStatus>,
    address: String,
    public_key: Pubkey,
    mut before_signature: Option<Signature>,
    until_signature: Option<Signature>,
    fetched_signatures: usize,
    fetched_transactions: usize,
) -> Result<(), AppError> {
    let mut total_signs = fetched_signatures;
    let mut total_txns = fetched_transactions;
    let mut batch = 1;
    const BATCH_SIZE: usize = 1000;

    loop {
        // Sleep for a while to avoid RPC rate limits
        tokio_sleep(100).await;

        // Get the next batch transaction signatures
        let signatures = state
            .rpc
            .get_signatures_for_address_with_config(
                &public_key,
                GetConfirmedSignaturesForAddress2Config {
                    before: before_signature,
                    until: until_signature,
                    limit: Some(BATCH_SIZE),
                    commitment: None,
                },
            )
            .await?;

        if signatures.is_empty() {
            event!(Level::INFO, "No more transactions found");
            break;
        }
        before_signature = Some(Signature::from_str(
            &signatures.last().unwrap().signature.clone(),
        )?);

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
                indexed_at: bson_current_time(),
            });
        }

        // Insert the transaction signatures into DB
        insert_transactions_signatures(&state.db, &txn_signs).await?;

        total_signs += signatures.len();

        // Get the total transaction signatures count of the account in DB
        let sign_count = get_signatures_count(&state.db, &address).await?;

        // Send the transaction signatures data status to the channel
        send_sync_message(
            &sender,
            SyncStatus::TransactionSignatures(serde_json::to_string(&TotalFetch {
                total: sign_count,
                fetched: total_signs as u64,
            })?),
        )
        .await;

        let mut txns: Vec<Transaction> = vec![];
        for sign in &signatures {
            let signature = Signature::from_str(&sign.signature)?;

            // Sleep again since these are individual RPC calls to get each transaction
            tokio_sleep(100).await;

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
                indexed_at: bson_current_time(),
            });
        }

        // Insert the transactions into DB
        insert_transactions(&state.db, &txns).await?;

        total_txns += txns.len();

        // Get the total transactions count of the account in DB
        let txn_count = get_transactions_count(&state.db, &address).await?;

        // Send the transaction signatures data status to the channel
        send_sync_message(
            &sender,
            SyncStatus::TransactionDetails(serde_json::to_string(&TotalFetch {
                total: txn_count,
                fetched: total_txns as u64,
            })?),
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
    }

    // Once the indexing/syncing/refreshing is completed
    // set the address indexing state to Idle
    update_address_indexing_state(
        &state.db,
        &address,
        UpdateAddressIndexingState {
            state: IndexingState::Idle,
            updated_at: bson_current_time(),
        },
    )
    .await?;
    event!(Level::INFO, "Indexing is completed");

    // Send the completed indexing message to the channel
    send_sync_message(&sender, SyncStatus::Completed).await;

    Ok(())
}

pub async fn refresher(
    state: AppState,
    sender: broadcast::Sender<SyncStatus>,
    address: String,
) -> Result<(), AppError> {
    // Convert the address str to Address struct instance of Solana account
    let public_key = Pubkey::from_str(&address)?;

    // You can only refresh an indexed account
    if !check_account_exists(&state.db, &address).await {
        return Err(AppError::BadRequest("Account is not indexed".to_string()));
    }

    // Set the address indexing state to Syncing
    update_address_indexing_state(
        &state.db,
        &address,
        UpdateAddressIndexingState {
            state: IndexingState::Syncing,
            updated_at: bson_current_time(),
        },
    )
    .await?;

    // Get the Solana account data of the address
    let account = state.rpc.get_account(&public_key).await?;
    event!(Level::INFO, ?account);

    // Update the account data in DB with the latest data
    let updated = update_account(
        &state.db,
        &address,
        UpdateAccount {
            lamports: account.lamports as i64,
            owner: account.owner.to_string(),
            executable: account.executable,
            data_length: account.data.len() as i64,
            rent_epoch: account.rent_epoch as i64,
            last_updated_at: bson_current_time(),
        },
    )
    .await?;

    // Send the updated account data to the channel
    send_sync_message(
        &sender,
        SyncStatus::AccountData(serde_json::to_string(&updated)?),
    )
    .await;

    // Get the latest signature to continue the sync/refresh
    let latest_signature = get_latest_signature(&state.db, address.clone()).await?;
    event!(Level::INFO, ?latest_signature);

    continue_sync(
        state,
        sender,
        address,
        public_key,
        None,
        Some(Signature::from_str(&latest_signature)?),
        0,
        0,
    )
    .await?;

    Ok(())
}
