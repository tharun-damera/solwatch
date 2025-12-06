use futures::stream::TryStreamExt;
use mongodb::{
    Database,
    bson::{Document, doc, from_document},
    options::{FindOneAndUpdateOptions, ReturnDocument},
};
use tracing::{Level, event};

use crate::error::AppError;
use crate::models::{Account, UpdateAccount};

const ACCOUNTS: &str = "accounts";

pub async fn get_account(db: &Database, address: &str) -> Result<Option<Account>, AppError> {
    let account = db
        .collection::<Account>(ACCOUNTS)
        .find_one(doc! {"_id": address})
        .await?;

    Ok(account)
}

pub async fn check_account_exists(db: &Database, address: &str) -> bool {
    match get_account(db, address).await {
        Ok(acc) => match acc {
            Some(account) => {
                event!(Level::INFO, "Account Found: {account:?}");
                true
            }
            None => {
                event!(Level::INFO, "Account Not Found");
                false
            }
        },
        Err(e) => {
            event!(Level::ERROR, "Error occurred while finding account: {e:?}");
            false
        }
    }
}

pub async fn insert_account(db: &Database, account: &Account) -> Result<(), AppError> {
    let inserted = db
        .collection::<Account>(ACCOUNTS)
        .insert_one(account)
        .await?;
    event!(Level::INFO, ?inserted);

    Ok(())
}

pub async fn update_account(
    db: &Database,
    address: &str,
    account: UpdateAccount,
) -> Result<Account, AppError> {
    // option to return the document after the update
    let options = FindOneAndUpdateOptions::builder()
        .return_document(ReturnDocument::After)
        .build();

    // Find the account in DB
    // If it exists update it and return the updated document
    // Else return Account not found error
    db.collection::<Account>(ACCOUNTS)
        .find_one_and_update(
            doc! {"_id": address},
            doc! {"$set": {
                "lamports": account.lamports,
                "owner": account.owner,
                "executable": account.executable,
                "data_length": account.data_length,
                "rent_epoch": account.rent_epoch,
                "last_updated_at": account.last_updated_at,
            }},
        )
        .with_options(options)
        .await?
        .ok_or_else(|| AppError::NotFound("Account Not Found".into()))
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct IndexerStats {
    account_exists: bool,
    signatures: i64,
    transactions: i64,
}

pub async fn get_indexer_stats(db: &Database, address: &str) -> Result<IndexerStats, AppError> {
    let docs: Vec<Document> = db
        .collection::<Document>(ACCOUNTS)
        .aggregate([
            doc! {
                "$match": {
                    "_id": address
                }
            },
            doc! {
                "$lookup": {
                    "from": "transaction_signatures",
                    "localField": "_id",
                    "foreignField": "account_address",
                    "as": "signatures",
                    "pipeline": [
                        {
                            "$count": "count"
                        }
                    ]
                }
            },
            doc! {
                "$lookup": {
                    "from": "transactions",
                    "localField": "_id",
                    "foreignField": "account_address",
                    "as": "transactions",
                    "pipeline": [
                        {
                            "$count": "count"
                        }
                    ]
                }
            },
            doc! {
                "$addFields": {
                    "account_exists": true
                }
            },
            doc! {
                "$project": {
                    "_id": 0,
                    "account_exists": 1,
                    "signatures": {
                        "$ifNull": [
                            {
                                "$first": "$signatures.count"
                            },
                            0
                        ]
                    },
                    "transactions": {
                        "$ifNull": [
                            {
                                "$first": "$transactions.count"
                            },
                            0
                        ]
                    }
                }
            },
        ])
        .await?
        .try_collect()
        .await?;

    if let Some(doc) = docs.into_iter().next() {
        let decoded: IndexerStats = from_document(doc)?;
        event!(Level::INFO, ?decoded);
        Ok(decoded)
    } else {
        Err(AppError::NotFound("Account Not Found".into()))
    }
}
