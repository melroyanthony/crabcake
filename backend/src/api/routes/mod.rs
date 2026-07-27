pub mod health;

use axum::Router;

use crate::AppState;

/// Everything served under `/api/v1`.
pub fn router() -> Router<AppState> {
    Router::new()
}
