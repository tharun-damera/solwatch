use sqlx::{PgPool, postgres::PgPoolOptions};

use crate::error::AppError;

pub mod accounts;

pub async fn init() -> Result<PgPool, AppError> {
    // Get the Database URL from the env
    let db_url = std::env::var("DATABASE_URL")?;

    // Setup the connection pool for Postgres DB
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&db_url)
        .await?;

    // Perform the DB migrations
    sqlx::migrate!("./migrations").run(&pool).await?;

    Ok(pool)
}
