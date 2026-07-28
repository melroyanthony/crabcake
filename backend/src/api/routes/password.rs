use axum::{Json, extract::State};
use secrecy::{ExposeSecret, SecretString};
use serde::Deserialize;
use utoipa::ToSchema;
use utoipa_axum::{router::OpenApiRouter, routes};
use validator::Validate;

use crate::{
    AppResult, AppState,
    api::extract::ValidatedJson,
    error::Problem,
    models::{Message, Password},
    services,
};

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct RecoveryRequest {
    #[validate(email(message = "must be a valid email address"))]
    #[schema(format = Email, example = "ada@example.com")]
    pub email: String,
}

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct ResetRequest {
    /// The token from the emailed link.
    #[schema(value_type = String)]
    pub token: SecretString,
    #[validate(nested)]
    pub new_password: Password,
}

pub fn router() -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        .routes(routes!(recover))
        .routes(routes!(reset))
}

/// Request a password reset
///
/// Emails a single-use link that expires. Always answers the same way, whether or not the
/// address is registered, so that this cannot be used to find out who has an account.
#[utoipa::path(
    post,
    path = "/recover",
    tag = "password",
    request_body = RecoveryRequest,
    responses(
        (status = OK, description = "The request was accepted, registered or not", body = Message),
        (status = UNPROCESSABLE_ENTITY, body = Problem),
    )
)]
async fn recover(
    State(state): State<AppState>,
    ValidatedJson(payload): ValidatedJson<RecoveryRequest>,
) -> AppResult<Json<Message>> {
    services::password_reset::request(&state, &payload.email).await?;

    Ok(Json(Message::new(
        "if that address has an account, a reset link is on its way",
    )))
}

/// Reset a password
///
/// Consumes the token from the emailed link and ends every session that account had.
#[utoipa::path(
    post,
    path = "/reset",
    tag = "password",
    request_body = ResetRequest,
    responses(
        (status = OK, body = Message),
        (status = UNPROCESSABLE_ENTITY, description = "The link is invalid, used, or expired, or the password is too short", body = Problem),
    )
)]
async fn reset(
    State(state): State<AppState>,
    ValidatedJson(payload): ValidatedJson<ResetRequest>,
) -> AppResult<Json<Message>> {
    services::password_reset::confirm(&state, payload.token.expose_secret(), &payload.new_password)
        .await?;

    Ok(Json(Message::new("password reset")))
}
