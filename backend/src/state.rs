use std::{ops::Deref, sync::Arc};

use sqlx::PgPool;

use crate::{Config, jobs::EmailQueue, storage::Storage};

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
    storage: Storage,
}

impl AppState {
    pub fn new(config: Config, db: PgPool, emails: EmailQueue, storage: Storage) -> Self {
        Self {
            inner: Arc::new(Inner {
                config,
                db,
                emails,
                storage,
            }),
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

    pub fn storage(&self) -> &Storage {
        &self.inner.storage
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
