pub mod docs;
pub mod extract;
pub mod rate_limit;
pub mod routes;
pub mod trace;

use std::time::Duration;

use axum::{
    Json, Router,
    http::{HeaderValue, StatusCode},
    routing::get,
};
use tower::{Layer, ServiceBuilder};
use tower_http::{
    catch_panic::CatchPanicLayer,
    compression::CompressionLayer,
    cors::{Any, CorsLayer},
    limit::RequestBodyLimitLayer,
    normalize_path::NormalizePathLayer,
    request_id::{MakeRequestUuid, PropagateRequestIdLayer, SetRequestIdLayer},
    timeout::TimeoutLayer,
    trace::TraceLayer,
};
use utoipa::OpenApi;
use utoipa_axum::router::OpenApiRouter;
use utoipa_scalar::{Scalar, Servable};

use crate::{AppError, AppState, api::docs::ApiDoc};

/// Where the interactive documentation lives, and where the document behind it is served.
pub const DOCS_PATH: &str = "/docs";
pub const OPENAPI_PATH: &str = "/api/openapi.json";

/// The application as served.
///
/// Trailing slashes are trimmed so that `/api/v1/users/` reaches the same handler as
/// `/api/v1/users`. That has to happen outside the router, because a `Router` layer runs after
/// the path has been matched, and by then the trailing slash is already a 404.
///
/// The result is then wrapped in an otherwise empty router, which looks redundant but is not:
/// `axum::serve` can only attach each connection's address to a `Router`, and without that
/// address the rate limiter cannot tell callers apart when there is no proxy in front to say
/// who they are.
pub fn serve(state: AppState) -> Router {
    let normalized = NormalizePathLayer::trim_trailing_slash().layer(build(state));

    Router::new().fallback_service(normalized)
}

/// Assembles the router and the middleware every request passes through.
pub fn build(state: AppState) -> Router {
    let config = state.config();

    // Ordered outermost first: a panic deep in a handler is caught before it can take the
    // connection down, and the request id exists before anything tries to log it.
    let middleware = ServiceBuilder::new()
        .layer(CatchPanicLayer::new())
        .layer(SetRequestIdLayer::x_request_id(MakeRequestUuid))
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(trace::RequestSpan)
                .on_response(trace::RecordResponse),
        )
        .layer(PropagateRequestIdLayer::x_request_id())
        .layer(TimeoutLayer::with_status_code(
            StatusCode::REQUEST_TIMEOUT,
            Duration::from_secs(config.request_timeout_seconds),
        ))
        .layer(RequestBodyLimitLayer::new(config.body_limit_bytes))
        .layer(CompressionLayer::new())
        .layer(cors(config.cors_origins()));

    let (router, mut openapi) = OpenApiRouter::with_openapi(ApiDoc::openapi())
        .merge(routes::health::router())
        .nest("/api/v1", routes::router())
        .split_for_parts();

    // The title is only known at runtime, so the document carries a neutral one until here.
    openapi.info.title.clone_from(&config.project_name);

    let router = router
        // Served from the finished document rather than rebuilt per request, so what the
        // documentation shows is exactly what the router does.
        .merge(Scalar::with_url(DOCS_PATH, openapi.clone()))
        .route(OPENAPI_PATH, get(async move || Json(openapi.clone())))
        // Without this, an unmatched path returns a bodyless 404 while every other error is
        // problem+json, and clients need two ways to read a failure.
        .fallback(not_found)
        .with_state(state.clone());

    let config = state.config();

    // Both sit inside the middleware stack: a rejected request is still traced and still carries
    // a request id, and a request that is rate limited is still counted.
    let router = rate_limit::apply(router, config);
    let router = crate::telemetry::metrics::apply(router, config);

    router.layer(middleware)
}

/// The document on its own, for the binary that writes it to a file.
pub fn openapi() -> utoipa::openapi::OpenApi {
    let (_, openapi) = OpenApiRouter::<AppState>::with_openapi(ApiDoc::openapi())
        .merge(routes::health::router())
        .nest("/api/v1", routes::router())
        .split_for_parts();

    openapi
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
