use axum::{Json, extract::State};
use secrecy::{ExposeSecret, SecretString};
use serde::Deserialize;
use utoipa::ToSchema;
use utoipa_axum::{router::OpenApiRouter, routes};

use crate::{
    AppResult, AppState,
    auth::CurrentUser,
    error::Problem,
    models::{Message, UserPublic},
    services::auth::{self, TokenPair},
};

/// The password and the refresh token are `SecretString` so that deriving `Debug` on a request
/// body cannot put credentials into a log line.
#[derive(Debug, Deserialize, ToSchema)]
pub struct Credentials {
    #[schema(format = Email, example = "ada@example.com")]
    pub email: String,
    #[schema(value_type = String, format = Password)]
    pub password: SecretString,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct RefreshRequest {
    #[schema(value_type = String)]
    pub refresh_token: SecretString,
}

pub fn router() -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        .routes(routes!(access_token))
        .routes(routes!(refresh))
        .routes(routes!(logout))
        .routes(routes!(logout_everywhere))
        .routes(routes!(test_token))
}

/// Sign in
///
/// Exchanges an email and password for a token pair.
#[utoipa::path(
    post,
    path = "/access-token",
    tag = "login",
    request_body = Credentials,
    responses(
        (status = OK, body = TokenPair),
        (status = UNAUTHORIZED, description = "Unknown address or wrong password", body = Problem),
        (status = FORBIDDEN, description = "The account is deactivated", body = Problem),
    )
)]
async fn access_token(
    State(state): State<AppState>,
    Json(credentials): Json<Credentials>,
) -> AppResult<Json<TokenPair>> {
    let tokens = auth::login(
        &state,
        &credentials.email,
        credentials.password.expose_secret(),
    )
    .await?;

    Ok(Json(tokens))
}

/// Refresh a session
///
/// Exchanges a refresh token for a new pair, invalidating the one presented. A stolen token
/// is therefore good for at most one refresh, and the theft shows up as the real client
/// being signed out.
#[utoipa::path(
    post,
    path = "/refresh",
    tag = "login",
    request_body = RefreshRequest,
    responses(
        (status = OK, body = TokenPair),
        (status = UNAUTHORIZED, description = "Unknown, expired, or already used token", body = Problem),
    )
)]
async fn refresh(
    State(state): State<AppState>,
    Json(request): Json<RefreshRequest>,
) -> AppResult<Json<TokenPair>> {
    let tokens = auth::refresh(&state, request.refresh_token.expose_secret()).await?;

    Ok(Json(tokens))
}

/// Sign out
///
/// Ends the session belonging to the given refresh token. Unknown tokens succeed quietly:
/// signing out should never fail, and saying which tokens exist would leak information.
#[utoipa::path(
    post,
    path = "/logout",
    tag = "login",
    request_body = RefreshRequest,
    responses((status = OK, body = Message))
)]
async fn logout(
    State(state): State<AppState>,
    Json(request): Json<RefreshRequest>,
) -> AppResult<Json<Message>> {
    auth::logout(&state, request.refresh_token.expose_secret()).await?;

    Ok(Json(Message::new("signed out")))
}

/// Sign out everywhere
///
/// Ends every session the caller has, on every device.
#[utoipa::path(
    post,
    path = "/logout-everywhere",
    tag = "login",
    security(("bearer" = [])),
    responses(
        (status = OK, body = Message),
        (status = UNAUTHORIZED, body = Problem),
    )
)]
async fn logout_everywhere(
    State(state): State<AppState>,
    CurrentUser(user): CurrentUser,
) -> AppResult<Json<Message>> {
    let ended = auth::logout_everywhere(&state, user.id).await?;

    Ok(Json(Message::new(format!("ended {ended} sessions"))))
}

/// Test an access token
///
/// Confirms an access token is valid and returns who it belongs to.
#[utoipa::path(
    post,
    path = "/test-token",
    tag = "login",
    security(("bearer" = [])),
    responses(
        (status = OK, body = UserPublic),
        (status = UNAUTHORIZED, body = Problem),
    )
)]
async fn test_token(CurrentUser(user): CurrentUser) -> Json<UserPublic> {
    Json(user.into())
}
