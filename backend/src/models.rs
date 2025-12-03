use mongodb::bson::DateTime as BsonDateTime;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Serialize, Deserialize)]
pub struct Account {
    #[serde(rename = "_id")]
    pub address: String,
    pub lamports: i64,
    pub owner: String,
    pub executable: bool,
    pub data_length: i64,
    pub rent_epoch: i64,
    pub indexed_at: BsonDateTime,
    pub last_updated_at: BsonDateTime,
}

#[derive(Debug)]
pub struct UpdateAccount {
    pub lamports: i64,
    pub owner: String,
    pub executable: bool,
    pub data_length: i64,
    pub rent_epoch: i64,
    pub last_updated_at: BsonDateTime,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TransactionSignature {
    #[serde(rename = "_id")]
    pub signature: String,
    pub account_address: String,
    pub slot: i64,
    pub block_time: Option<i64>,
    pub confirmation_status: String,
    pub indexed_at: BsonDateTime,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Transaction {
    #[serde(rename = "_id")]
    pub signature: String,
    pub account_address: String,
    pub slot: i64,
    pub block_time: Option<i64>,
    pub transaction: Value,
    pub indexed_at: BsonDateTime,
}
