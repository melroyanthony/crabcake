pub mod health;
pub mod items;
pub mod login;
pub mod users;

use axum::Router;

use crate::AppState;

/// Everything served under `/api/v1`.
pub fn router() -> Router<AppState> {
    Router::new()
        .nest("/login", login::router())
        .nest("/users", users::router())
        .nest("/items", items::router())
}
