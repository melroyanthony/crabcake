use apalis::prelude::{Data, Error, Monitor, Storage as _, WorkerBuilder, WorkerFactoryFn};
use apalis_sql::postgres::PostgresStorage;
use sqlx::PgPool;

use crate::{
    AppError, AppResult,
    email::{Email, Mailer},
};

/// The queue of outbound mail. Postgres-backed, so it needs no Redis and survives a restart,
/// and so enqueueing can be part of the same transactional story as the rest of the data.
pub type EmailQueue = PostgresStorage<Email>;

/// Creates the queue's own tables. Runs on startup alongside the schema migrations, so that a
/// fresh database is ready without a separate step.
///
/// Runs apalis's migrator by hand rather than calling its own `setup`, only so that the same
/// `set_ignore_missing` can be applied: both sets of migrations record themselves in
/// `_sqlx_migrations`, and neither should trip over the other's rows. See [`crate::db::migrate`].
pub async fn setup(pool: &PgPool) -> AppResult<()> {
    let mut migrator = PostgresStorage::migrations();
    migrator.set_ignore_missing(true);

    migrator.run(pool).await.map_err(|error| {
        AppError::Unexpected(anyhow::anyhow!("could not set up the queue: {error}"))
    })?;

    Ok(())
}

pub fn queue(pool: PgPool) -> EmailQueue {
    PostgresStorage::new(pool)
}

/// Hands a message to the queue. Cloning the storage is how apalis expects to be used from
/// shared state: pushing needs `&mut`, and the clone is a handle to the same tables.
pub async fn enqueue(queue: &EmailQueue, email: Email) -> AppResult<()> {
    let subject = email.subject.clone();
    let to = email.to.clone();

    queue.clone().push(email).await.map_err(|error| {
        AppError::Unexpected(anyhow::anyhow!("could not enqueue an email: {error}"))
    })?;

    tracing::info!(%to, %subject, "email queued");
    Ok(())
}

/// Runs until the process is stopped, sending whatever turns up.
pub async fn run_worker(queue: EmailQueue, mailer: Mailer) -> anyhow::Result<()> {
    Monitor::new()
        .register(
            WorkerBuilder::new("email")
                .data(mailer)
                .backend(queue)
                .build_fn(send_email),
        )
        .run()
        .await?;

    Ok(())
}

/// A failure here is returned rather than swallowed, so apalis retries it. That is the whole
/// reason mail goes through a queue: a mail server that is down for a minute should delay an
/// email, not lose it and not fail somebody's password reset.
async fn send_email(email: Email, mailer: Data<Mailer>) -> Result<(), Error> {
    mailer
        .send(&email)
        .await
        .map_err(|error| Error::Failed(std::sync::Arc::new(Box::new(error))))
}
