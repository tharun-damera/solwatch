mod db;
mod error;
mod handlers;
mod message;
mod models;
mod routes;
mod solana;
mod tracer;

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

    // Create an app router for handling requests
    // that takes in PgPool as state to perform DB operations
    let app = routes::create_router(pool);

    // Add a tcp binding to listen to requests at port 5000
    // Localhost (127.0.0.1) for Local
    // 0.0.0.0 for Prod
    let listener = tokio::net::TcpListener::bind("127.0.0.1:5000").await?;

    // Serve the app with the tcp listener
    axum::serve(listener, app).await?;

    Ok(())
}
