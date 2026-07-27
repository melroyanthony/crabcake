use std::{ops::Deref, sync::Arc};

use sqlx::PgPool;

use crate::Config;

/// Shared application state. Cloned for every request, so everything inside is either cheap
/// to clone or behind an `Arc`.
#[derive(Debug, Clone)]
pub struct AppState {
    inner: Arc<Inner>,
}

#[derive(Debug)]
struct Inner {
    config: Config,
    db: PgPool,
}

impl AppState {
    pub fn new(config: Config, db: PgPool) -> Self {
        Self {
            inner: Arc::new(Inner { config, db }),
        }
    }

    pub fn config(&self) -> &Config {
        &self.inner.config
    }

    pub fn db(&self) -> &PgPool {
        &self.inner.db
    }
}

impl Deref for AppState {
    type Target = Config;

    fn deref(&self) -> &Self::Target {
        self.config()
    }
}
