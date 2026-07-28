use std::time::Duration;

use axum::{
    extract::MatchedPath,
    http::{Request, Response, header},
};
use tower_http::trace::{MakeSpan, OnResponse};
use tracing::{Level, Span, field};

/// The span every request gets.
///
/// Created at INFO rather than the DEBUG that `tower_http` uses by default, because a span that
/// the default `RUST_LOG` filters out is a span that never reaches a collector, and nobody
/// enjoys discovering that their traces are empty because of a log filter.
#[derive(Clone, Copy)]
pub struct RequestSpan;

impl<B> MakeSpan<B> for RequestSpan {
    fn make_span(&mut self, request: &Request<B>) -> Span {
        // The matched route rather than the path, so that a million requests for different item
        // ids are one operation in a trace view instead of a million.
        let route = request
            .extensions()
            .get::<MatchedPath>()
            .map(MatchedPath::as_str)
            .unwrap_or_default();

        let request_id = request
            .headers()
            .get("x-request-id")
            .and_then(|value| value.to_str().ok())
            .unwrap_or_default();

        tracing::info_span!(
            "http.request",
            // What a trace viewer shows as the operation name. Without it, every span in the
            // system is called "http.request".
            otel.name = %format!("{} {}", request.method(), if route.is_empty() { request.uri().path() } else { route }),
            otel.kind = "server",
            otel.status_code = field::Empty,
            http.request.method = %request.method(),
            http.route = %route,
            url.path = %request.uri().path(),
            http.response.status_code = field::Empty,
            // Ties a trace to the log lines and to the header the client was given back.
            request_id = %request_id,
        )
    }
}

/// Records the outcome on the span the request opened.
#[derive(Clone, Copy)]
pub struct RecordResponse;

impl<B> OnResponse<B> for RecordResponse {
    fn on_response(self, response: &Response<B>, latency: Duration, span: &Span) {
        let status = response.status();

        span.record("http.response.status_code", status.as_u16());

        // Only a fault on our side marks the span as an error. A 404 or a 422 is the API doing
        // its job, and a trace view full of red for rejected input tells you nothing.
        if status.is_server_error() {
            span.record("otel.status_code", "ERROR");
        }

        // A request id is only useful if the client is given the same one, which the
        // request-id layer takes care of.
        let request_id = response
            .headers()
            .get(header::HeaderName::from_static("x-request-id"))
            .and_then(|value| value.to_str().ok())
            .unwrap_or_default();

        let level = if status.is_server_error() {
            Level::WARN
        } else {
            Level::DEBUG
        };

        // The one line that says what happened, at a level that depends on whether anybody
        // needs to care.
        match level {
            Level::WARN => tracing::warn!(
                status = status.as_u16(),
                latency_ms = latency.as_millis(),
                request_id,
                "request failed"
            ),
            _ => tracing::debug!(
                status = status.as_u16(),
                latency_ms = latency.as_millis(),
                request_id,
                "request finished"
            ),
        }
    }
}

#[cfg(test)]
mod tests {
    use axum::{Router, body::Body, routing::get};
    use tower::ServiceExt as _;
    use tower_http::trace::TraceLayer;

    use super::*;

    /// The span has to survive a request without panicking on a missing matched path, which is
    /// what happens for anything that does not match a route.
    #[tokio::test]
    async fn an_unmatched_path_still_gets_a_span() {
        let router = Router::new().route("/known", get(async || "ok")).layer(
            TraceLayer::new_for_http()
                .make_span_with(RequestSpan)
                .on_response(RecordResponse),
        );

        let response = router
            .oneshot(
                Request::builder()
                    .uri("/unknown")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), 404);
    }
}
