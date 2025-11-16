use axum::{
    Router,
    routing::{any, get},
};

use crate::{AppState, cors::setup_cors_layer, handlers::*};

pub fn create_router(state: AppState) -> Router {
    // Setup the cors layer and add it to the router
    let cors_layer = setup_cors_layer();

    // Setup a router consisting of the routes with the Connection pool as State accessible to all the handlers
    Router::new()
        .route("/ws/index/{address}", any(websocket_handler))
        .route("/api/accounts/{address}/status", get(get_account_status))
        .route("/api/accounts/{address}", get(get_account_data))
        .route(
            "/api/accounts/{address}/signatures",
            get(transaction_signatures),
        )
        .route("/api/accounts/{address}/transactions", get(transactions))
        .route(
            "/api/accounts/{address}/transactions/{signature}",
            get(transaction_from_signature),
        )
        .with_state(state)
        .layer(cors_layer)
}
