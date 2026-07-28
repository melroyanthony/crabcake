use std::sync::OnceLock;

use axum::{Router, routing::get};
use axum_prometheus::{
    PrometheusMetricLayerBuilder, metrics_exporter_prometheus::PrometheusHandle,
};

use crate::Config;

/// Holding the handle here rather than passing it around mirrors how the crate works: installing
/// the exporter registers a process-wide recorder, so there can only ever be one, and the
/// listener needs to reach it from a different part of startup than the layer.
static HANDLE: OnceLock<PrometheusHandle> = OnceLock::new();

/// Adds request metrics to the router when `METRICS_ENABLED` is set, and nothing otherwise.
pub fn apply(router: Router, config: &Config) -> Router {
    if !config.metrics_enabled {
        return router;
    }

    // Paths are grouped by their route rather than their literal value, so a million requests to
    // /api/v1/items/{id} are one series instead of a million.
    let (layer, handle) = PrometheusMetricLayerBuilder::new()
        .with_prefix("app")
        .with_default_metrics()
        .build_pair();

    if HANDLE.set(handle).is_err() {
        tracing::warn!("metrics were already installed; ignoring the second attempt");
    }

    router.layer(layer)
}

/// The router that serves the metrics, if any were installed.
///
/// Served on its own listener rather than alongside the API: request counts per route, and the
/// paths themselves, describe the shape of a system to anyone who asks, and an endpoint that is
/// never routed from outside cannot be forgotten about.
pub fn router() -> Option<Router> {
    let handle = HANDLE.get()?.clone();

    Some(Router::new().route(
        "/metrics",
        get(move || {
            let handle = handle.clone();
            async move { handle.render() }
        }),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nothing_is_installed_when_metrics_are_off() {
        let config = Config {
            metrics_enabled: false,
            ..Config::for_tests()
        };

        // Applying to an empty router and getting one back is all there is to observe here; the
        // recorder is global, so a test that installed it would leak into every other test.
        let _ = apply(Router::new(), &config);

        assert!(router().is_none());
    }
}
