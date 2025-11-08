use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::FromRow;

#[derive(Debug, FromRow, Serialize)]
pub struct Account {
    pub address: String,
    pub lamports: i64,
    pub owner: String,
    pub executable: bool,
    pub data_length: i64,
    pub rent_epoch: i64,
    pub indexed_at: DateTime<Utc>,
    pub last_updated_at: DateTime<Utc>,
}

#[derive(Debug, FromRow, Serialize)]
pub struct TransactionSignature {
    pub signature: String,
    pub account_address: String,
    pub slot: i64,
    pub block_time: Option<i64>,
    pub confirmation_status: String,
    pub indexed_at: DateTime<Utc>,
}

#[derive(Debug)]
pub struct AccountCreate {
    pub address: String,
    pub lamports: i64,
    pub owner: String,
    pub executable: bool,
    pub data_length: i64,
    pub rent_epoch: i64,
}

#[derive(Debug)]
pub struct TransactionSignatureCreate {
    pub signature: String,
    pub account_address: String,
    pub slot: i64,
    pub block_time: Option<i64>,
    pub confirmation_status: String,
}
