use std::net::SocketAddr;

use anyhow::Context;
use app::{AppState, Config, api, bootstrap, db, jobs, storage::Storage, telemetry};
use tokio::{net::TcpListener, signal};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();

    let config = Config::from_env()?;
    let telemetry = telemetry::init(&config);

    let pool = db::connect(&config).await?;
    db::migrate(&pool).await?;
    jobs::setup(&pool).await?;
    bootstrap::ensure_first_superuser(&pool, &config).await?;

    let address = config.bind_address;
    let metrics_address = config.metrics_bind_address;
    let emails = jobs::queue(pool.clone());
    let storage = Storage::from_config(&config).await;
    let state = AppState::new(config, pool, emails, storage);

    // Builds the router first, because that is what installs the metrics recorder and so decides
    // whether there is anything for the metrics listener to serve.
    let app = api::serve(state);

    if let Some(metrics) = telemetry::metrics::router() {
        serve_metrics(metrics, metrics_address).await?;
    }

    let listener = TcpListener::bind(address)
        .await
        .with_context(|| format!("could not bind to {address}"))?;

    tracing::info!(%address, "listening");

    // With connect info, so that the rate limiter can identify a caller that reaches the API
    // directly rather than through a proxy that names them in a header.
    let result = axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .with_graceful_shutdown(shutdown_signal())
    .await
    .context("server error");

    // After the server has stopped, so that spans from the requests it was still finishing are
    // included rather than dropped.
    telemetry.shutdown();

    result
}

/// Serves metrics on their own listener, in the background.
///
/// Binding happens here rather than in the spawned task so that a port already in use is a
/// startup error, not a warning nobody reads.
async fn serve_metrics(router: axum::Router, address: SocketAddr) -> anyhow::Result<()> {
    let listener = TcpListener::bind(address)
        .await
        .with_context(|| format!("could not bind the metrics listener to {address}"))?;

    tracing::info!(%address, "serving metrics");

    tokio::spawn(async move {
        if let Err(error) = axum::serve(listener, router).await {
            tracing::error!(%error, "the metrics listener stopped");
        }
    });

    Ok(())
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
