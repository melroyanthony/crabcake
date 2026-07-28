use axum::{Json, extract::State};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use utoipa_axum::{router::OpenApiRouter, routes};
use validator::Validate;

use crate::{
    AppError, AppResult, AppState, api::extract::ValidatedJson, auth::CurrentUser, error::Problem,
    models::Message, storage::keys,
};

/// What the client is about to upload. The name is only used to build a readable key, and the
/// content type is signed into the URL, so both are advisory rather than trusted.
#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct UploadRequest {
    #[validate(length(min = 1, max = 255, message = "must be 1 to 255 characters"))]
    #[schema(example = "holiday-photo.jpg")]
    pub filename: String,
    #[validate(length(min = 1, max = 255, message = "must be 1 to 255 characters"))]
    #[schema(example = "image/jpeg")]
    pub content_type: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct UploadTarget {
    /// Where the file will live. Keep it: this is what you store against your own records.
    #[schema(example = "uploads/1b4e28ba-.../9f8c.../holiday-photo.jpg")]
    pub key: String,
    /// `PUT` the file here, with the same `Content-Type` that was asked for.
    pub url: String,
    /// How long the URL is good for.
    #[schema(example = 900)]
    pub expires_in_seconds: u64,
}

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct KeyRequest {
    #[validate(length(min = 1, max = 1024, message = "must be 1 to 1024 characters"))]
    pub key: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct DownloadLink {
    pub url: String,
    #[schema(example = 900)]
    pub expires_in_seconds: u64,
}

pub fn router() -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        .routes(routes!(create))
        .routes(routes!(link))
        .routes(routes!(delete))
}

/// Start an upload
///
/// Answers with a URL to `PUT` the file to. The file goes straight to storage rather than
/// through the API, so its size is limited by the bucket rather than by this server.
#[utoipa::path(
    post,
    path = "",
    tag = "uploads",
    request_body = UploadRequest,
    security(("bearer" = [])),
    responses(
        (status = OK, body = UploadTarget),
        (status = UNAUTHORIZED, body = Problem),
        (status = UNPROCESSABLE_ENTITY, body = Problem),
        (status = NOT_IMPLEMENTED, description = "This deployment has no bucket configured", body = Problem),
    )
)]
async fn create(
    State(state): State<AppState>,
    CurrentUser(user): CurrentUser,
    ValidatedJson(payload): ValidatedJson<UploadRequest>,
) -> AppResult<Json<UploadTarget>> {
    let storage = state.storage();
    let key = keys::for_user(user.id, &payload.filename);
    let url = storage.presigned_put(&key, &payload.content_type).await?;

    tracing::info!(user = %user.id, %key, "handed out an upload URL");

    Ok(Json(UploadTarget {
        key,
        url,
        expires_in_seconds: storage.expires_in().as_secs(),
    }))
}

/// Get a download link
///
/// Works only for your own files, so a key guessed or copied from somewhere else is a 404
/// rather than a way to read another account's uploads.
#[utoipa::path(
    post,
    path = "/link",
    tag = "uploads",
    request_body = KeyRequest,
    security(("bearer" = [])),
    responses(
        (status = OK, body = DownloadLink),
        (status = UNAUTHORIZED, body = Problem),
        (status = NOT_FOUND, description = "No such file, or not yours", body = Problem),
        (status = NOT_IMPLEMENTED, body = Problem),
    )
)]
async fn link(
    State(state): State<AppState>,
    CurrentUser(user): CurrentUser,
    ValidatedJson(payload): ValidatedJson<KeyRequest>,
) -> AppResult<Json<DownloadLink>> {
    let storage = state.storage();
    owned(&state, &payload.key, &user).await?;

    Ok(Json(DownloadLink {
        url: storage.presigned_get(&payload.key).await?,
        expires_in_seconds: storage.expires_in().as_secs(),
    }))
}

/// Delete a file
#[utoipa::path(
    delete,
    path = "",
    tag = "uploads",
    request_body = KeyRequest,
    security(("bearer" = [])),
    responses(
        (status = OK, body = Message),
        (status = UNAUTHORIZED, body = Problem),
        (status = NOT_FOUND, body = Problem),
        (status = NOT_IMPLEMENTED, body = Problem),
    )
)]
async fn delete(
    State(state): State<AppState>,
    CurrentUser(user): CurrentUser,
    ValidatedJson(payload): ValidatedJson<KeyRequest>,
) -> AppResult<Json<Message>> {
    owned(&state, &payload.key, &user).await?;
    state.storage().delete(&payload.key).await?;

    tracing::info!(user = %user.id, key = %payload.key, "deleted an upload");

    Ok(Json(Message::new("deleted")))
}

/// Checks that a key is the caller's and that something is actually stored under it.
///
/// A key that belongs to somebody else is reported as missing rather than forbidden, so that
/// this cannot be used to find out which keys exist. Superusers are not exempt: nothing here
/// needs them to be, and "an administrator can read anyone's files" is a decision for whoever
/// uses this template, not a default to inherit.
async fn owned(state: &AppState, key: &str, user: &crate::models::User) -> AppResult<()> {
    if !keys::belongs_to(key, user.id) {
        tracing::warn!(user = %user.id, %key, "refused a key belonging to someone else");
        return Err(AppError::NotFound);
    }

    if state.storage().exists(key).await? {
        Ok(())
    } else {
        Err(AppError::NotFound)
    }
}
