use bson::DateTime;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Account {
    #[serde(rename = "_id")]
    pub address: String,
    pub lamports: i64,
    pub owner: String,
    pub executable: bool,
    pub data_length: i64,
    pub rent_epoch: i64,
    pub indexed_at: DateTime,
    pub last_updated_at: DateTime,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TransactionSignature {
    #[serde(rename = "_id")]
    pub signature: String,
    pub account_address: String,
    pub slot: i64,
    pub block_time: Option<i64>,
    pub confirmation_status: String,
    pub indexed_at: DateTime,
}
