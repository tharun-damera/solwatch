use tracing::{info, instrument};

#[instrument]
#[tokio::main]
async fn main() -> Result<(), backend::error::AppError> {
    // Load the variables from the .env file as env variables
    dotenvy::dotenv().ok();

    // Setup tracing/logging and get the WorkerGuard that flushes logs periodically
    // This guard has to live in the entry point of the program (i.e. main fn)
    // Lives as long as the main fn
    let _guard = backend::tracer::setup_tracing();

    // Build the app that initiates the DB, connects to Solana RPC and includes them in the
    // app state for the axum route handlers
    let app = backend::build_app().await?;

    // Get the host and port from env
    let host = std::env::var("APP_HOST").expect("APP_HOST env variable is mising");
    let port = std::env::var("APP_PORT").expect("APP_PORT env variable is mising");

    let bind = format!("{}:{}", host, port);
    info!("[+] Server running on {bind:?}...");

    // Add a tcp binding to listen to requests at the configured host and port
    let listener = tokio::net::TcpListener::bind(bind).await?;

    // Serve the app with the tcp listener
    axum::serve(listener, app).await?;

    Ok(())
}
