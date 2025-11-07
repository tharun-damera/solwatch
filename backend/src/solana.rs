use std::str::FromStr;

use axum::extract::ws::{Message, WebSocket};
use solana_client::rpc_client::RpcClient;
use solana_sdk::pubkey::Pubkey;
use sqlx::PgPool;

use crate::db::accounts::insert_account;
use crate::error::AppError;
use crate::models::SolAccount;

const DEV_NET: &str = "https://api.devnet.solana.com";

pub async fn index_address(
    mut socket: WebSocket,
    pool: &PgPool,
    address: String,
) -> Result<(), AppError> {
    let connection = RpcClient::new(DEV_NET);
    let public_key = Pubkey::from_str(address.as_str())?;
    dbg!(&public_key);

    let account = connection.get_account(&public_key)?;
    dbg!(&account);

    let inserted_acc = insert_account(
        pool,
        SolAccount {
            address,
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

    Ok(())
}
