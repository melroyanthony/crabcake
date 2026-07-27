use axum::{Json, Router, extract::State, routing::get};
use serde::Serialize;

use crate::{AppResult, AppState};

#[derive(Debug, Serialize)]
pub struct Health {
    status: &'static str,
    version: &'static str,
}

impl Health {
    fn ok() -> Self {
        Self {
            status: "ok",
            version: env!("CARGO_PKG_VERSION"),
        }
    }
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/health", get(live))
        .route("/health/ready", get(ready))
}

/// Liveness: the process is up. Deliberately touches nothing else, so a database blip cannot
/// convince an orchestrator to restart a perfectly healthy process.
async fn live() -> Json<Health> {
    Json(Health::ok())
}

/// Readiness: the process can actually serve traffic, which means reaching the database.
async fn ready(State(state): State<AppState>) -> AppResult<Json<Health>> {
    sqlx::query("SELECT 1").execute(state.db()).await?;
    Ok(Json(Health::ok()))
}
