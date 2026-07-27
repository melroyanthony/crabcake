use anyhow::Context;
use app::{AppState, Config, api, bootstrap, db, telemetry};
use tokio::{net::TcpListener, signal};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();

    let config = Config::from_env()?;
    telemetry::init(&config);

    let pool = db::connect(&config).await?;
    db::migrate(&pool).await?;
    bootstrap::ensure_first_superuser(&pool, &config).await?;

    let address = config.bind_address;
    let state = AppState::new(config, pool);
    let app = api::build(state);

    let listener = TcpListener::bind(address)
        .await
        .with_context(|| format!("could not bind to {address}"))?;

    tracing::info!(%address, "listening");

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .context("server error")
}

/// Stops accepting new connections on Ctrl-C or SIGTERM, letting in-flight requests finish.
/// Without the SIGTERM arm, every container stop would be a hard kill.
async fn shutdown_signal() {
    let interrupt = async {
        signal::ctrl_c().await.expect("failed to listen for Ctrl-C");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to listen for SIGTERM")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        () = interrupt => {},
        () = terminate => {},
    }

    tracing::info!("shutting down");
}
