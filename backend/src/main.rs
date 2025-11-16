use std::sync::Arc;

use mongodb::{Client, Database};
use solana_client::nonblocking::rpc_client::RpcClient;
use tracing::{Level, event, instrument};

mod cors;
mod db;
mod error;
mod handlers;
mod message;
mod models;
mod routes;
mod solana;
mod tracer;

// Solana Devnet RPC URL
const DEV_NET: &str = "https://api.devnet.solana.com";

#[derive(Clone)]
pub struct AppState {
    pub client: Client,
    pub db: Database,
    pub rpc: Arc<RpcClient>,
}

#[instrument(skip_all)]
#[tokio::main]
async fn main() -> Result<(), error::AppError> {
    // Load the variables from the .env file as env variables
    dotenvy::dotenv().ok();

    // Setup tracing/logging and get the WorkerGuard that flushes logs periodically
    // This guard has to live in the entry point of the program (i.e. main fn)
    // Lives as long as the main fn
    let _guard = tracer::setup_tracing();

    // Setup Mongo Client and Database
    let (client, db) = db::init().await?;

    // Connect to the Solana Devnet through RPC (Remote Procedure Call)
    let rpc = Arc::new(RpcClient::new(DEV_NET.to_string()));

    // Create an AppState containing Mongo Client, Database and RpcClient
    let state = AppState { client, db, rpc };

    // Create an app router for handling requests
    // that takes in the AppState to perform DB operations & RPC calls
    let app = routes::create_router(state);

    // Get the host and port from env
    let host = std::env::var("APP_HOST")?;
    let port = std::env::var("APP_PORT")?;
    let bind = format!("{}:{}", host, port);
    event!(Level::INFO, "[+] Server running on {bind:?}...");

    // Add a tcp binding to listen to requests at the configured host and port
    let listener = tokio::net::TcpListener::bind(bind).await?;

    // Serve the app with the tcp listener
    axum::serve(listener, app).await?;

    Ok(())
}
