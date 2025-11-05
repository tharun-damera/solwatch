use sqlx::{PgPool, postgres::PgPoolOptions};

use crate::error::AppError;

pub mod accounts;

pub async fn init() -> Result<PgPool, AppError> {
    let db_url = std::env::var("DATABASE_URL")?;
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&db_url)
        .await?;

    sqlx::migrate!("./migrations").run(&pool).await?;
    Ok(pool)
}
