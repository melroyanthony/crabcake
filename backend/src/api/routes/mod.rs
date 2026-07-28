pub mod health;
pub mod items;
pub mod login;
pub mod users;

use utoipa_axum::router::OpenApiRouter;

use crate::AppState;

/// Everything served under `/api/v1`.
pub fn router() -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        .nest("/login", login::router())
        .nest("/users", users::router())
        .nest("/items", items::router())
}
