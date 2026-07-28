use std::sync::Arc;

use axum::{
    Router,
    body::Body,
    http::{HeaderValue, Response, StatusCode, header},
};
use tower_governor::{
    GovernorLayer, errors::GovernorError, governor::GovernorConfigBuilder,
    key_extractor::SmartIpKeyExtractor,
};

use crate::{Config, error::Problem};

/// Wraps a router in a per-caller rate limit, or returns it untouched when
/// `RATE_LIMIT_PER_SECOND` is 0.
///
/// Applied to the router rather than added to the middleware stack so that the outer layers
/// still run: a caller who is turned away gets a request id and leaves a trace explaining why.
///
/// The limit is per caller, so one noisy client cannot lock everyone else out. State lives in
/// this process, so two API replicas allow twice the configured rate, which is the usual trade
/// for keeping Redis out of the request path.
pub fn apply(router: Router, config: &Config) -> Router {
    if config.rate_limit_per_second == 0 {
        tracing::info!("rate limiting is off");
        return router;
    }

    // `SmartIpKeyExtractor` reads X-Forwarded-For and X-Real-IP before falling back to the
    // socket address, which is what makes this work behind Traefik. It also means whatever sits
    // in front must overwrite those headers rather than pass a caller's own through, or a caller
    // can pick a fresh identity per request.
    let governor = GovernorConfigBuilder::default()
        .per_second(u64::from(config.rate_limit_per_second))
        .burst_size(config.rate_limit_burst)
        .key_extractor(SmartIpKeyExtractor)
        .finish()
        // Only fails for a zero period or burst, and zero is handled above.
        .expect("the rate limit is valid");

    tracing::info!(
        per_second = config.rate_limit_per_second,
        burst = config.rate_limit_burst,
        "rate limiting is on"
    );

    router.layer(GovernorLayer::new(Arc::new(governor)).error_handler(problem_response))
}

/// Answers a rejected request the way every other failure is answered, rather than with the
/// crate's plain-text default, so a client needs only one way to read an error.
fn problem_response(error: GovernorError) -> Response<Body> {
    let (status, detail, retry_after) = match error {
        GovernorError::TooManyRequests { wait_time, .. } => (
            StatusCode::TOO_MANY_REQUESTS,
            format!("too many requests, try again in {wait_time}s"),
            Some(wait_time),
        ),
        GovernorError::UnableToExtractKey => (
            StatusCode::INTERNAL_SERVER_ERROR,
            "could not identify the caller".to_owned(),
            None,
        ),
        GovernorError::Other { code, msg, .. } => (
            code,
            msg.unwrap_or_else(|| "rate limiting failed".to_owned()),
            None,
        ),
    };

    let problem = Problem {
        status: status.as_u16(),
        title: status.canonical_reason().unwrap_or("Error").to_owned(),
        detail,
    };

    let body = serde_json::to_vec(&problem).unwrap_or_default();

    let mut response = Response::new(Body::from(body));
    *response.status_mut() = status;
    response.headers_mut().insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("application/problem+json"),
    );

    // Tells a well-behaved client when to come back instead of leaving it to guess.
    if let Some(seconds) = retry_after
        && let Ok(value) = HeaderValue::from_str(&seconds.to_string())
    {
        response.headers_mut().insert(header::RETRY_AFTER, value);
    }

    response
}

#[cfg(test)]
mod tests {
    use axum::{body::to_bytes, http::Request, routing::get};
    use tower::ServiceExt as _;

    use super::*;

    fn router(per_second: u32, burst: u32) -> Router {
        let config = Config {
            rate_limit_per_second: per_second,
            rate_limit_burst: burst,
            ..Config::for_tests()
        };

        apply(Router::new().route("/", get(async || "ok")), &config)
    }

    async fn call(router: &Router, caller: &str) -> Response<Body> {
        let request = Request::builder()
            .uri("/")
            .header("x-real-ip", caller)
            .body(Body::empty())
            .expect("the request is valid");

        router
            .clone()
            .oneshot(request)
            .await
            .expect("the router answers")
    }

    #[tokio::test]
    async fn a_caller_over_the_limit_is_turned_away() {
        let router = router(1, 1);

        assert_eq!(call(&router, "203.0.113.1").await.status(), StatusCode::OK);

        let rejected = call(&router, "203.0.113.1").await;

        assert_eq!(rejected.status(), StatusCode::TOO_MANY_REQUESTS);
        assert_eq!(
            rejected.headers().get(header::CONTENT_TYPE).unwrap(),
            "application/problem+json"
        );
        assert!(rejected.headers().contains_key(header::RETRY_AFTER));

        let body = to_bytes(rejected.into_body(), 4096).await.unwrap();
        let problem: serde_json::Value = serde_json::from_slice(&body).unwrap();

        assert_eq!(problem["status"], 429);
        assert!(
            problem["detail"]
                .as_str()
                .unwrap()
                .contains("too many requests")
        );
    }

    /// The whole point of keying by caller: one client burning through its allowance must not
    /// take everyone else down with it.
    #[tokio::test]
    async fn one_caller_hitting_the_limit_does_not_affect_another() {
        let router = router(1, 1);

        assert_eq!(call(&router, "203.0.113.1").await.status(), StatusCode::OK);
        assert_eq!(
            call(&router, "203.0.113.1").await.status(),
            StatusCode::TOO_MANY_REQUESTS
        );

        assert_eq!(call(&router, "203.0.113.2").await.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn zero_lets_everything_through() {
        let router = router(0, 0);

        for _ in 0..5 {
            assert_eq!(call(&router, "203.0.113.1").await.status(), StatusCode::OK);
        }
    }
}
