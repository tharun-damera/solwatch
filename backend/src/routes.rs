use axum::{Router, routing::get};
use tower::ServiceBuilder;
use tower_http::{
    request_id::{MakeRequestUuid, PropagateRequestIdLayer, SetRequestIdLayer},
    trace::{DefaultMakeSpan, DefaultOnRequest, DefaultOnResponse, TraceLayer},
};
use tracing::Level;

use crate::{AppState, cors::setup_cors_layer, handlers::*};

pub fn create_router(state: AppState) -> Router {
    // Setup the cors layer and add it to the router
    let cors_layer = setup_cors_layer();

    // Build a combined layer / middleware and add it to the Router
    // Requests (Layers are called from Top -> Bottom)
    // Responses (Layers are called from Bottom -> Top)
    let combined_layer = ServiceBuilder::new()
        // Layer that generates a request_id for every HTTP request
        .layer(SetRequestIdLayer::x_request_id(MakeRequestUuid))
        // Layer that propogates the x-request-id header downstream (i.e. the header is added in response)
        .layer(PropagateRequestIdLayer::x_request_id())
        // Layer that logs request and response information
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(DefaultMakeSpan::new().level(Level::INFO))
                .on_request(DefaultOnRequest::new().level(Level::INFO))
                .on_response(DefaultOnResponse::new().level(Level::INFO)),
        )
        // Layer that handles the CORS
        .layer(cors_layer);

    // Setup a router consisting of the routes with the Connection pool as State accessible to all the handlers
    Router::new()
        // SSE (Server Sent Event) route for indexing
        .route("/api/accounts/{address}/index/sse", get(indexer_sse))
        // Remaining API routes
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
        .route("/api/accounts/{address}/refresh/sse", get(refresh_sse))
        // Application state
        .with_state(state)
        // Add the layer / middleware at the end
        .layer(combined_layer)
}
