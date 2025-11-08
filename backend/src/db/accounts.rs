use sqlx::{PgPool, QueryBuilder};

use crate::error::AppError;
use crate::models::{Account, AccountCreate, TransactionSignatureCreate};

pub async fn get_account(pool: &PgPool, address: String) -> Result<Account, AppError> {
    let account = sqlx::query_as!(Account, "SELECT * FROM accounts WHERE address=$1", address)
        .fetch_one(pool)
        .await?;
    Ok(account)
}

pub async fn check_account_exists(pool: &PgPool, address: String) -> bool {
    let account = get_account(pool, address).await;
    match account {
        Ok(acc) => {
            println!("{:?}", acc);
            true
        }
        Err(e) => {
            println!("Error: {:?}", e);
            false
        }
    }
}

pub async fn insert_account(pool: &PgPool, account: AccountCreate) -> Result<Account, AppError> {
    let account = sqlx::query_as!(
        Account,
        "INSERT INTO accounts (
            address,
            lamports,
            owner,
            executable,
            data_length,
            rent_epoch
        ) VALUES (
            $1,
            $2,
            $3,
            $4,
            $5,
            $6
        ) 
        RETURNING *",
        account.address,
        account.lamports,
        account.owner,
        account.executable,
        account.data_length,
        account.rent_epoch,
    )
    .fetch_one(pool)
    .await?;
    dbg!(&account);

    Ok(account)
}

pub async fn insert_transactions(
    pool: &PgPool,
    signatures: &[TransactionSignatureCreate],
) -> Result<(), AppError> {
    let mut qb = QueryBuilder::new(
        "INSERT INTO transaction_signatures (
            signature,
            account_address,
            slot,
            block_time,
            confirmation_status
        ) ",
    );
    qb.push_values(signatures.iter(), |mut b, txn| {
        b.push_bind(&txn.signature)
            .push_bind(&txn.account_address)
            .push_bind(txn.slot)
            .push_bind(txn.block_time)
            .push_bind(&txn.confirmation_status);
    });
    qb.push(";");

    let query = qb.build();

    let txn_signs = query.execute(pool).await?;
    dbg!(&txn_signs);

    Ok(())
}
