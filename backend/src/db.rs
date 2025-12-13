use mongodb::{Client, Database};

use crate::error::AppError;

pub mod accounts;
pub mod transactions;

pub async fn init() -> Result<Database, AppError> {
    // Get the Mongo URI and DB name from the env
    let uri = std::env::var("MONGO_URI")?;
    let db = std::env::var("MONGO_DB")?;

    // Setup the Mongo Database
    let db = Client::with_uri_str(uri).await?.database(&db);

    Ok(db)
}
