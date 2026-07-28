//! Drains the job queue.
//!
//! A separate process from the API, so that a slow mail server cannot make requests slow, and
//! so workers can be scaled without scaling the API.

use anyhow::Context as _;
use app::{Config, db, email::Mailer, jobs, telemetry};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();

    let config = Config::from_env()?;
    telemetry::init(&config);

    let pool = db::connect(&config).await?;

    // The API also does this on startup. Doing it here too means the worker can be the first
    // thing to come up without failing on tables that do not exist yet.
    jobs::setup(&pool).await?;

    let mailer = Mailer::from_config(&config)?;

    tracing::info!("worker started");

    jobs::run_worker(jobs::queue(pool), mailer)
        .await
        .context("worker stopped unexpectedly")
}
