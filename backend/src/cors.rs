use axum::http::header::HeaderValue;
use tower_http::cors::{Any, CorsLayer};

pub fn setup_cors_layer() -> CorsLayer {
    // Get the allowed origins list from env
    let allowed_origins_str =
        std::env::var("ALLOWED_ORIGINS").expect("ALLOWED_ORIGINS must be set in .env");

    // Convert the origins string to a vector of HeaderValue
    let allowed_origins: Vec<HeaderValue> = allowed_origins_str
        .split(",")
        .map(|s| s.trim().parse::<HeaderValue>().unwrap())
        .collect();

    // Create a CorsLayer that
    CorsLayer::new()
        .allow_origin(allowed_origins) // allows the specific origins
        .allow_methods(Any) // allows all HTTP methods
        .allow_headers(Any) // allows all HTTP headers
}
