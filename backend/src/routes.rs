use axum::{
    Router,
    routing::{any, get},
};

use crate::handlers::*;

pub fn create_router() -> Router {
    Router::new()
        .route("/ws", any(websocket_handler))
        .route("/api/account/{address}/status", get(get_account_status))
}
