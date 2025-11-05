use sqlx::PgPool;

use crate::error::AppError;
use crate::models::Account;

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
