use sqlx::PgPool;

use crate::error::AppError;
use crate::models::{Account, SolAccount};

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

pub async fn insert_account(pool: &PgPool, account: SolAccount) -> Result<Account, AppError> {
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
