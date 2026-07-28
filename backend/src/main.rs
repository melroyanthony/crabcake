use anyhow::Context;
use app::{AppState, Config, api, bootstrap, db, jobs, telemetry};
use axum::{ServiceExt, extract::Request};
use tokio::{net::TcpListener, signal};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();

    let config = Config::from_env()?;
    telemetry::init(&config);

    let pool = db::connect(&config).await?;
    db::migrate(&pool).await?;
    jobs::setup(&pool).await?;
    bootstrap::ensure_first_superuser(&pool, &config).await?;

    let address = config.bind_address;
    let emails = jobs::queue(pool.clone());
    let state = AppState::new(config, pool, emails);
    let app = api::serve(state);

    let listener = TcpListener::bind(address)
        .await
        .with_context(|| format!("could not bind to {address}"))?;

    tracing::info!(%address, "listening");

    // into_make_service_with_connect_info is not used here, so the plain make service is what
    // the path-normalising wrapper needs in order to be served.
    axum::serve(listener, ServiceExt::<Request>::into_make_service(app))
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
