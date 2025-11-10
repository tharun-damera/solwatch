use std::sync::Arc;

use solana_client::nonblocking::rpc_client::RpcClient;

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

#[tokio::main]
async fn main() -> Result<(), error::AppError> {
    // Load the variables from the .env file as env variables
    dotenvy::dotenv().ok();

    // Setup tracing/logging and get the WorkerGuard that flushes logs periodically
    // This guard has to live in the entry point of the program (i.e. main fn)
    // Lives as long as the main fn
    let _guard = tracer::setup_tracing();

    // Initiate a Postgres connection pool
    let pool = db::init().await?;

    // Connect to the Solana Devnet through RPC (Remote Procedure Call)
    let rpc = Arc::new(RpcClient::new(DEV_NET.to_string()));

    // Create an AppState containing PgPool and RpcClient
    let state = routes::AppState { pool, rpc };

    // Create an app router for handling requests
    // that takes in the AppState to perform DB operations & RPC calls
    let app = routes::create_router(state);

    // Add a tcp binding to listen to requests at port 5000
    // Localhost (127.0.0.1) for Local
    // 0.0.0.0 for Prod
    let listener = tokio::net::TcpListener::bind("127.0.0.1:5000").await?;

    // Serve the app with the tcp listener
    axum::serve(listener, app).await?;

    Ok(())
}
