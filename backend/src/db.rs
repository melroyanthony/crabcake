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
///
/// The job queue brings its own migrations, and sqlx 0.8 offers no way to give either set its
/// own version table, so the two share `_sqlx_migrations`. Each therefore has to tolerate rows
/// it did not write, which is what `set_ignore_missing` means here. The version spaces cannot
/// collide: these migrations are numbered from 1 while the queue's are timestamps.
pub async fn migrate(pool: &PgPool) -> anyhow::Result<()> {
    let mut migrator = sqlx::migrate!("./migrations");
    migrator.set_ignore_missing(true);

    migrator.run(pool).await?;

    tracing::info!("migrations are up to date");
    Ok(())
}
