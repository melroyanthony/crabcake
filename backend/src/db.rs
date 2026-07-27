use std::time::Duration;

use sqlx::postgres::{PgPool, PgPoolOptions};

use crate::Config;

/// Opens the connection pool. Connecting lazily would let the process start while the
/// database is unreachable, which turns a startup failure into a confusing 500 later.
pub async fn connect(config: &Config) -> anyhow::Result<PgPool> {
    let pool = PgPoolOptions::new()
        .max_connections(20)
        .acquire_timeout(Duration::from_secs(5))
        .connect(&config.database_url)
        .await?;

    tracing::info!("connected to the database");
    Ok(pool)
}

/// Applies any migrations the database has not seen yet.
pub async fn migrate(pool: &PgPool) -> anyhow::Result<()> {
    sqlx::migrate!("./migrations").run(pool).await?;
    tracing::info!("migrations are up to date");
    Ok(())
}
