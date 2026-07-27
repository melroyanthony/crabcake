pub mod routes;

use std::time::Duration;

use axum::{
    Router,
    http::{HeaderValue, StatusCode},
};
use tower::ServiceBuilder;
use tower_http::{
    catch_panic::CatchPanicLayer,
    compression::CompressionLayer,
    cors::{Any, CorsLayer},
    limit::RequestBodyLimitLayer,
    request_id::{MakeRequestUuid, PropagateRequestIdLayer, SetRequestIdLayer},
    timeout::TimeoutLayer,
    trace::TraceLayer,
};

use crate::{AppError, AppState};

/// Assembles the router and the middleware every request passes through.
pub fn build(state: AppState) -> Router {
    let config = state.config();

    // Ordered outermost first: a panic deep in a handler is caught before it can take the
    // connection down, and the request id exists before anything tries to log it.
    let middleware = ServiceBuilder::new()
        .layer(CatchPanicLayer::new())
        .layer(SetRequestIdLayer::x_request_id(MakeRequestUuid))
        .layer(TraceLayer::new_for_http())
        .layer(PropagateRequestIdLayer::x_request_id())
        .layer(TimeoutLayer::with_status_code(
            StatusCode::REQUEST_TIMEOUT,
            Duration::from_secs(config.request_timeout_seconds),
        ))
        .layer(RequestBodyLimitLayer::new(config.body_limit_bytes))
        .layer(CompressionLayer::new())
        .layer(cors(config.cors_origins()));

    Router::new()
        .merge(routes::health::router())
        .nest("/api/v1", routes::router())
        // Without this, an unmatched path returns a bodyless 404 while every other error is
        // problem+json, and clients need two ways to read a failure.
        .fallback(not_found)
        .layer(middleware)
        .with_state(state)
}

async fn not_found() -> AppError {
    AppError::NotFound
}

fn cors(origins: Vec<&str>) -> CorsLayer {
    let layer = CorsLayer::new().allow_methods(Any).allow_headers(Any);

    let parsed: Vec<HeaderValue> = origins
        .iter()
        .filter_map(|origin| match HeaderValue::from_str(origin) {
            Ok(value) => Some(value),
            Err(_) => {
                tracing::warn!(origin, "ignoring malformed CORS origin");
                None
            }
        })
        .collect();

    if parsed.is_empty() {
        layer
    } else {
        layer.allow_origin(parsed)
    }
}
