mod db;
mod error;
mod handlers;
mod models;
mod routes;
mod solana;

#[tokio::main]
async fn main() -> Result<(), error::AppError> {
    dotenvy::dotenv().ok();

    let pool = db::init().await?;
    let app = routes::create_router(pool);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:5000").await?;
    axum::serve(listener, app).await?;

    Ok(())
}
