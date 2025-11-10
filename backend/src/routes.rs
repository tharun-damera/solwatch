use axum::{
    Router,
    routing::{any, get},
};
use sqlx::PgPool;

use crate::handlers::*;

pub fn create_router(pool: PgPool) -> Router {
    // Setup a router consisting of the routes with the Connection pool as State accessible to all the handlers
    Router::new()
        .route("/ws/index/{address}", any(websocket_handler))
        .route("/api/account/{address}/status", get(get_account_status))
        .with_state(pool)
}
