pub mod metrics;
pub mod otel;

use tracing_subscriber::{EnvFilter, fmt, prelude::*};

use crate::config::{Config, Environment};

/// What has to be shut down when the process stops. Dropping it is not enough: batched spans
/// sit in a buffer, and without an explicit flush the last few seconds of a trace are simply
/// lost, which is the part you usually wanted.
pub struct Guard {
    tracer: Option<otel::TracerProvider>,
}

impl Guard {
    /// Flushes anything still buffered. Called on graceful shutdown.
    pub fn shutdown(self) {
        if let Some(tracer) = self.tracer {
            otel::shutdown(tracer);
        }
    }
}

/// Sets up logging, and trace export when it is configured.
///
/// Locally logging means readable, colourised lines; everywhere else it means one JSON object
/// per line, which is what log aggregators expect. Spans are exported over OTLP only when
/// `OTEL_EXPORTER_OTLP_ENDPOINT` is set, so this stays quiet on a laptop with no collector.
pub fn init(config: &Config) -> Guard {
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info,app=debug,tower_http=debug,sqlx=warn"));

    // Boxed so that both shapes are one type. Without that, the OpenTelemetry layer below would
    // have to be built separately for each branch, since its type is tied to the subscriber it
    // is added to.
    let logs = match config.environment {
        Environment::Local => fmt::layer().pretty().boxed(),
        Environment::Staging | Environment::Production => {
            fmt::layer().json().flatten_event(true).boxed()
        }
    };

    // Built before the subscriber is installed, because a failure here has to be reported
    // through `eprintln!`: there is nowhere else for it to go yet.
    let tracer = otel::tracer_provider(config);

    let traces = tracer
        .as_ref()
        .map(|provider| tracing_opentelemetry::layer().with_tracer(otel::tracer(provider)));

    tracing_subscriber::registry()
        .with(filter)
        .with(logs)
        .with(traces)
        .init();

    if tracer.is_some() {
        tracing::info!(
            endpoint = %config.otel_exporter_otlp_endpoint,
            service = %config.otel_service_name(),
            "exporting traces"
        );
    }

    Guard { tracer }
}
