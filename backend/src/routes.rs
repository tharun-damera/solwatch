use std::sync::Arc;

use axum::{
    Router,
    routing::{any, get},
};
use solana_client::nonblocking::rpc_client::RpcClient;
use sqlx::PgPool;

use crate::handlers::*;

#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
    pub rpc: Arc<RpcClient>,
}

pub fn create_router(state: AppState) -> Router {
    // Setup a router consisting of the routes with the Connection pool as State accessible to all the handlers
    Router::new()
        .route("/ws/index/{address}", any(websocket_handler))
        .route("/api/account/{address}/status", get(get_account_status))
        .with_state(state)
}
