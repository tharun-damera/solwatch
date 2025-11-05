mod db;
mod error;
mod handlers;
mod models;
mod routes;

#[tokio::main]
async fn main() -> Result<(), error::AppError> {
    dotenvy::dotenv().ok();

    let pool = db::init().await?;
    let app = routes::create_router(pool);

    let listener = tokio::net::TcpListener::bind("localhost:5000").await?;
    axum::serve(listener, app).await?;

    Ok(())
}
