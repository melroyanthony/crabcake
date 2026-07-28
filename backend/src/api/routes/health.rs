use axum::{Json, extract::State};
use serde::Serialize;
use utoipa::ToSchema;
use utoipa_axum::{router::OpenApiRouter, routes};

use crate::{AppResult, AppState, error::Problem};

#[derive(Debug, Serialize, ToSchema)]
pub struct Health {
    #[schema(example = "ok")]
    status: &'static str,
    #[schema(example = "0.1.0")]
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

pub fn router() -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        .routes(routes!(live))
        .routes(routes!(ready))
}

/// Liveness probe
///
/// Reports that the process is up. Deliberately touches nothing else, so a database blip
/// cannot convince an orchestrator to restart a perfectly healthy process.
#[utoipa::path(
    get,
    path = "/health",
    tag = "health",
    responses((status = OK, body = Health))
)]
async fn live() -> Json<Health> {
    Json(Health::ok())
}

/// Readiness probe
///
/// Reports that the process can actually serve traffic, which means reaching the database.
#[utoipa::path(
    get,
    path = "/health/ready",
    tag = "health",
    responses(
        (status = OK, body = Health),
        (status = INTERNAL_SERVER_ERROR, description = "The database is unreachable", body = Problem),
    )
)]
async fn ready(State(state): State<AppState>) -> AppResult<Json<Health>> {
    sqlx::query("SELECT 1").execute(state.db()).await?;
    Ok(Json(Health::ok()))
}
