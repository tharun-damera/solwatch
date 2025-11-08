use std::str::FromStr;

use axum::extract::ws::{Message, WebSocket};
use solana_client::rpc_client::RpcClient;
use solana_sdk::pubkey::Pubkey;
use sqlx::PgPool;

use crate::db::accounts::{insert_account, insert_transactions};
use crate::error::AppError;
use crate::models::{AccountCreate, TransactionSignatureCreate};

const DEV_NET: &str = "https://api.devnet.solana.com";

pub async fn index_address(
    mut socket: WebSocket,
    pool: &PgPool,
    address: String,
) -> Result<(), AppError> {
    while let Some(Ok(msg)) = socket.recv().await {
        dbg!(&msg);

        let connection = RpcClient::new(DEV_NET);
        let public_key = Pubkey::from_str(address.as_str())?;
        dbg!(&public_key);

        let account = connection.get_account(&public_key)?;
        dbg!(&account);

        let inserted_acc = insert_account(
            pool,
            AccountCreate {
                address: address.clone(),
                lamports: account.lamports as i64,
                owner: account.owner.to_string(),
                executable: account.executable,
                data_length: account.data.len() as i64,
                rent_epoch: account.rent_epoch as i64,
            },
        )
        .await?;

        match serde_json::to_string(&inserted_acc) {
            Ok(json_str) => {
                if let Err(e) = socket.send(Message::Text(json_str.into())).await {
                    eprintln!("Failed to send message: {}", e);
                }
            }
            Err(e) => {
                eprintln!("Failed to serialize account: {}", e);
            }
        }

        let signatures = connection.get_signatures_for_address(&public_key)?;
        dbg!(&signatures);

        let mut txn_signs: Vec<TransactionSignatureCreate> = vec![];

        for sign in signatures {
            txn_signs.push(TransactionSignatureCreate {
                signature: sign.signature,
                account_address: address.clone(),
                slot: sign.slot as i64,
                block_time: sign.block_time,
                confirmation_status: serde_json::from_str(&serde_json::to_string(
                    &sign.confirmation_status,
                )?)?,
            });
        }
        dbg!(&txn_signs);

        insert_transactions(pool, &txn_signs).await?;
    }
    Ok(())
}
