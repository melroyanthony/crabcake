use axum::{Json, Router, extract::State, routing::post};
use serde::Deserialize;

use crate::{
    AppResult, AppState,
    auth::CurrentUser,
    models::UserPublic,
    services::auth::{self, TokenPair},
};

#[derive(Debug, Deserialize)]
pub struct Credentials {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Deserialize)]
pub struct RefreshRequest {
    pub refresh_token: String,
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/access-token", post(access_token))
        .route("/refresh", post(refresh))
        .route("/logout", post(logout))
        .route("/logout-everywhere", post(logout_everywhere))
        .route("/test-token", post(test_token))
}

/// Exchanges an email and password for a token pair.
async fn access_token(
    State(state): State<AppState>,
    Json(credentials): Json<Credentials>,
) -> AppResult<Json<TokenPair>> {
    let tokens = auth::login(&state, &credentials.email, &credentials.password).await?;
    Ok(Json(tokens))
}

/// Exchanges a refresh token for a new pair, invalidating the one presented.
async fn refresh(
    State(state): State<AppState>,
    Json(request): Json<RefreshRequest>,
) -> AppResult<Json<TokenPair>> {
    let tokens = auth::refresh(&state, &request.refresh_token).await?;
    Ok(Json(tokens))
}

/// Ends the session belonging to the given refresh token.
async fn logout(
    State(state): State<AppState>,
    Json(request): Json<RefreshRequest>,
) -> AppResult<Json<Acknowledged>> {
    auth::logout(&state, &request.refresh_token).await?;
    Ok(Json(Acknowledged::new()))
}

/// Ends every session the caller has, on every device.
async fn logout_everywhere(
    State(state): State<AppState>,
    CurrentUser(user): CurrentUser,
) -> AppResult<Json<Acknowledged>> {
    auth::logout_everywhere(&state, user.id).await?;
    Ok(Json(Acknowledged::new()))
}

/// Confirms an access token is valid and returns who it belongs to.
async fn test_token(CurrentUser(user): CurrentUser) -> Json<UserPublic> {
    Json(user.into())
}

#[derive(Debug, serde::Serialize)]
pub struct Acknowledged {
    message: &'static str,
}

impl Acknowledged {
    fn new() -> Self {
        Self { message: "ok" }
    }
}
