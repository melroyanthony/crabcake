use std::{ops::Deref, sync::Arc};

use sqlx::PgPool;

use crate::{Config, jobs::EmailQueue};

/// Shared application state. Cloned for every request, so everything inside is either cheap
/// to clone or behind an `Arc`.
#[derive(Clone)]
pub struct AppState {
    inner: Arc<Inner>,
}

struct Inner {
    config: Config,
    db: PgPool,
    emails: EmailQueue,
}

impl AppState {
    pub fn new(config: Config, db: PgPool, emails: EmailQueue) -> Self {
        Self {
            inner: Arc::new(Inner { config, db, emails }),
        }
    }

    pub fn config(&self) -> &Config {
        &self.inner.config
    }

    pub fn db(&self) -> &PgPool {
        &self.inner.db
    }

    pub fn emails(&self) -> &EmailQueue {
        &self.inner.emails
    }
}

impl Deref for AppState {
    type Target = Config;

    fn deref(&self) -> &Self::Target {
        self.config()
    }
}

/// Written out rather than derived, because `EmailQueue` is not `Debug` and, more to the point,
/// printing the state should never be a way to find out what is in the configuration.
impl std::fmt::Debug for AppState {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.debug_struct("AppState").finish_non_exhaustive()
    }
}
