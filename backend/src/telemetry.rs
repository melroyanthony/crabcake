use tracing_subscriber::{EnvFilter, fmt, prelude::*};

use crate::config::{Config, Environment};

/// Sets up logging. Locally that means readable, colourised lines; everywhere else it means
/// one JSON object per line, which is what log aggregators expect.
pub fn init(config: &Config) {
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info,app=debug,tower_http=debug,sqlx=warn"));

    let registry = tracing_subscriber::registry().with(filter);

    match config.environment {
        Environment::Local => registry.with(fmt::layer().pretty()).init(),
        Environment::Staging | Environment::Production => registry
            .with(fmt::layer().json().flatten_event(true))
            .init(),
    }
}
