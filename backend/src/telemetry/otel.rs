use opentelemetry::trace::TracerProvider as _;
use opentelemetry_otlp::WithExportConfig as _;
use opentelemetry_sdk::{Resource, trace::SdkTracerProvider};

use crate::Config;

pub type TracerProvider = SdkTracerProvider;

/// Builds a tracer provider, or nothing when no collector is configured.
///
/// A failure to build the exporter is logged rather than fatal. Traces are how you find out why
/// a request was slow; they are not worth refusing to serve requests over.
pub fn tracer_provider(config: &Config) -> Option<TracerProvider> {
    if config.otel_exporter_otlp_endpoint.is_empty() {
        return None;
    }

    let exporter = opentelemetry_otlp::SpanExporter::builder()
        .with_tonic()
        .with_endpoint(&config.otel_exporter_otlp_endpoint)
        .build();

    match exporter {
        Ok(exporter) => Some(
            SdkTracerProvider::builder()
                .with_batch_exporter(exporter)
                // Without a service name every span arrives as "unknown_service", and a
                // collector receiving several services cannot tell them apart.
                .with_resource(
                    Resource::builder()
                        .with_service_name(config.otel_service_name().to_owned())
                        .build(),
                )
                .build(),
        ),
        Err(error) => {
            // The subscriber does not exist yet, so this cannot go through `tracing`.
            eprintln!(
                "could not build the OTLP exporter, continuing without trace export: {error}"
            );
            None
        }
    }
}

pub fn tracer(provider: &TracerProvider) -> opentelemetry_sdk::trace::SdkTracer {
    provider.tracer("app")
}

/// Flushes buffered spans. Anything still in the batch when the process exits is lost, and that
/// is usually the request that made someone look.
pub fn shutdown(provider: TracerProvider) {
    if let Err(error) = provider.shutdown() {
        tracing::warn!(%error, "could not flush traces on shutdown");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_endpoint_means_no_exporter() {
        assert!(tracer_provider(&Config::for_tests()).is_none());
    }
}
