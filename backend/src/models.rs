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

pub struct SolAccount {
    pub address: String,
    pub lamports: i64,
    pub owner: String,
    pub executable: bool,
    pub data_length: i64,
    pub rent_epoch: i64,
}
