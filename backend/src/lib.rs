use std::sync::Arc;

use solana_client::nonblocking::rpc_client::RpcClient;

pub mod app_state;
pub mod cors;
pub mod db;
pub mod error;
pub mod handlers;
pub mod message;
pub mod models;
pub mod routes;
pub mod solana;
pub mod tracer;

// Solana Devnet RPC URL
const DEV_NET: &str = "https://api.devnet.solana.com";
// Solana Mainnet RPC URL
const _MAIN_NET: &str = "https://api.mainnet-beta.solana.com";

pub async fn build_app() -> Result<axum::Router, error::AppError> {
    // Setup Mongo Database
    let db = db::init().await?;

    // Connect to the Solana Devnet through RPC (Remote Procedure Call)
    let rpc = Arc::new(RpcClient::new(DEV_NET.to_string()));

    // Create an AppState containing Mongo Database and RpcClient
    let state = app_state::AppState::new(db, rpc);

    // Create an app router for handling requests
    // that takes in the AppState to perform DB operations & RPC calls
    let app = routes::create_router(state);

    Ok(app)
}
