pub mod health;
pub mod login;

use axum::Router;

use crate::AppState;

/// Everything served under `/api/v1`.
pub fn router() -> Router<AppState> {
    Router::new().nest("/login", login::router())
}
