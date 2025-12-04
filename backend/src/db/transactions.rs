use futures::stream::TryStreamExt;
use mongodb::{Database, bson::doc, options::FindOneOptions};

use crate::error::AppError;
use crate::models::{Transaction, TransactionSignature};

const SIGNATURE_COLLECTION: &str = "transaction_signatures";
const TRANSACTION_COLLECTION: &str = "transactions";

pub async fn insert_transactions_signatures(
    db: &Database,
    signatures: &[TransactionSignature],
) -> Result<(), AppError> {
    db.collection::<TransactionSignature>(SIGNATURE_COLLECTION)
        .insert_many(signatures)
        .await?;
    Ok(())
}

pub async fn insert_transactions(db: &Database, txns: &[Transaction]) -> Result<(), AppError> {
    db.collection::<Transaction>(TRANSACTION_COLLECTION)
        .insert_many(txns)
        .await?;
    Ok(())
}

pub async fn get_transaction_signatures(
    db: &Database,
    address: String,
    skip: u64,
    limit: i64,
) -> Result<Vec<TransactionSignature>, AppError> {
    let signatures: Vec<TransactionSignature> = db
        .collection::<TransactionSignature>(SIGNATURE_COLLECTION)
        .find(doc! {"account_address": address})
        .sort(doc! {"slot": -1})
        .skip(skip)
        .limit(limit)
        .await?
        .try_collect()
        .await?;

    Ok(signatures)
}

pub async fn get_transactions(
    db: &Database,
    address: String,
    skip: u64,
    limit: i64,
) -> Result<Vec<Transaction>, AppError> {
    let signatures: Vec<Transaction> = db
        .collection::<Transaction>(TRANSACTION_COLLECTION)
        .find(doc! {"account_address": address})
        .sort(doc! {"slot": -1})
        .skip(skip)
        .limit(limit)
        .await?
        .try_collect()
        .await?;

    Ok(signatures)
}

pub async fn get_transaction(
    db: &Database,
    address: String,
    signature: String,
) -> Result<Option<Transaction>, AppError> {
    let signature = db
        .collection::<Transaction>(TRANSACTION_COLLECTION)
        .find_one(doc! {"_id": signature, "account_address": address})
        .await?;

    Ok(signature)
}

#[derive(serde::Deserialize)]
struct SignatureOnly {
    _id: String,
}

pub async fn get_latest_signature(db: &Database, address: String) -> Result<String, AppError> {
    let options = FindOneOptions::builder()
        .sort(doc! {"slot": -1})
        .projection(doc! {"_id": 1})
        .build();

    let latest_record = db
        .collection::<SignatureOnly>(SIGNATURE_COLLECTION)
        .find_one(doc! {"account_address": address})
        .with_options(options)
        .await?;

    latest_record
        .map(|r| Ok(r._id))
        .unwrap_or_else(|| Err(AppError::NotFoundError("Latest Signature".to_string())))
}

pub async fn get_signatures_count(db: &Database, address: &str) -> Result<u64, AppError> {
    let count = db
        .collection::<TransactionSignature>(SIGNATURE_COLLECTION)
        .count_documents(doc! {
            "account_address": address
        })
        .await?;

    Ok(count)
}

pub async fn get_transactions_count(db: &Database, address: &str) -> Result<u64, AppError> {
    let count = db
        .collection::<TransactionSignature>(SIGNATURE_COLLECTION)
        .count_documents(doc! {
            "account_address": address
        })
        .await?;

    Ok(count)
}
