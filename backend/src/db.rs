use mongodb::{Client, Database};

use crate::error::AppError;

pub mod accounts;

pub async fn init() -> Result<(Client, Database), AppError> {
    // Get the Mongo URI and DB name from the env
    let uri = std::env::var("MONGO_URI")?;
    let db = std::env::var("MONGO_DB")?;

    // Setup the Mongo Client
    let client = Client::with_uri_str(uri).await?;
    let db = client.database(&db);

    Ok((client, db))
}
