use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use utoipa_axum::{router::OpenApiRouter, routes};
use uuid::Uuid;

use crate::{
    AppError, AppResult, AppState,
    api::extract::{ValidatedJson, ValidatedQuery},
    auth::{CurrentUser, Superuser},
    error::Problem,
    models::{
        Message, Page, Pagination, PasswordUpdate, UserCreate, UserPublic, UserRegister,
        UserUpdate, UserUpdateMe,
    },
    repo, services,
};

pub fn router() -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        .routes(routes!(list, create))
        .routes(routes!(signup))
        .routes(routes!(read_me, update_me, delete_me))
        .routes(routes!(update_my_password))
        .routes(routes!(read, update, delete))
}

/// List users
///
/// Superusers only.
#[utoipa::path(
    get,
    path = "/",
    tag = "users",
    params(Pagination),
    security(("bearer" = [])),
    responses(
        (status = OK, body = Page<UserPublic>),
        (status = UNAUTHORIZED, body = Problem),
        (status = FORBIDDEN, description = "Not a superuser", body = Problem),
        (status = UNPROCESSABLE_ENTITY, description = "Invalid paging", body = Problem),
    )
)]
async fn list(
    State(state): State<AppState>,
    Superuser(_admin): Superuser,
    ValidatedQuery(page): ValidatedQuery<Pagination>,
) -> AppResult<Json<Page<UserPublic>>> {
    let users = repo::users::list(state.db(), page.skip, page.limit).await?;
    let count = repo::users::count(state.db()).await?;

    Ok(Json(Page::new(
        users.into_iter().map(UserPublic::from).collect(),
        count,
    )))
}

/// Create a user
///
/// Sets any combination of fields, including privileges. Superusers only.
#[utoipa::path(
    post,
    path = "/",
    tag = "users",
    request_body = UserCreate,
    security(("bearer" = [])),
    responses(
        (status = CREATED, body = UserPublic),
        (status = UNAUTHORIZED, body = Problem),
        (status = FORBIDDEN, description = "Not a superuser", body = Problem),
        (status = CONFLICT, description = "The address is taken", body = Problem),
        (status = UNPROCESSABLE_ENTITY, body = Problem),
    )
)]
async fn create(
    State(state): State<AppState>,
    Superuser(_admin): Superuser,
    ValidatedJson(payload): ValidatedJson<UserCreate>,
) -> AppResult<(StatusCode, Json<UserPublic>)> {
    let user = services::users::create(state.db(), payload).await?;

    Ok((StatusCode::CREATED, Json(user.into())))
}

/// Register
///
/// Open to anyone. The resulting account is always an ordinary active user, whatever the
/// request body asks for.
#[utoipa::path(
    post,
    path = "/signup",
    tag = "users",
    request_body = UserRegister,
    responses(
        (status = CREATED, body = UserPublic),
        (status = CONFLICT, description = "The address is taken", body = Problem),
        (status = UNPROCESSABLE_ENTITY, body = Problem),
    )
)]
async fn signup(
    State(state): State<AppState>,
    ValidatedJson(payload): ValidatedJson<UserRegister>,
) -> AppResult<(StatusCode, Json<UserPublic>)> {
    let user = services::users::register(state.db(), payload).await?;

    Ok((StatusCode::CREATED, Json(user.into())))
}

/// Read your own account
#[utoipa::path(
    get,
    path = "/me",
    tag = "users",
    security(("bearer" = [])),
    responses(
        (status = OK, body = UserPublic),
        (status = UNAUTHORIZED, body = Problem),
    )
)]
async fn read_me(CurrentUser(user): CurrentUser) -> Json<UserPublic> {
    Json(user.into())
}

/// Update your own account
///
/// Every field is optional; anything left out keeps its current value.
#[utoipa::path(
    patch,
    path = "/me",
    tag = "users",
    request_body = UserUpdateMe,
    security(("bearer" = [])),
    responses(
        (status = OK, body = UserPublic),
        (status = UNAUTHORIZED, body = Problem),
        (status = CONFLICT, description = "The address is taken", body = Problem),
        (status = UNPROCESSABLE_ENTITY, body = Problem),
    )
)]
async fn update_me(
    State(state): State<AppState>,
    CurrentUser(user): CurrentUser,
    ValidatedJson(payload): ValidatedJson<UserUpdateMe>,
) -> AppResult<Json<UserPublic>> {
    let updated = services::users::update_me(state.db(), user.id, payload).await?;

    Ok(Json(updated.into()))
}

/// Change your own password
///
/// Requires the current password, and ends every other session on success.
#[utoipa::path(
    patch,
    path = "/me/password",
    tag = "users",
    request_body = PasswordUpdate,
    security(("bearer" = [])),
    responses(
        (status = OK, body = Message),
        (status = UNAUTHORIZED, body = Problem),
        (status = UNPROCESSABLE_ENTITY, description = "Wrong current password, or a new one that is unchanged or too short", body = Problem),
    )
)]
async fn update_my_password(
    State(state): State<AppState>,
    CurrentUser(user): CurrentUser,
    ValidatedJson(payload): ValidatedJson<PasswordUpdate>,
) -> AppResult<Json<Message>> {
    services::users::change_password(state.db(), &user, payload).await?;

    Ok(Json(Message::new("password updated")))
}

/// Close your own account
///
/// Superusers cannot, since deleting the last administrator would leave no way back in.
#[utoipa::path(
    delete,
    path = "/me",
    tag = "users",
    security(("bearer" = [])),
    responses(
        (status = OK, body = Message),
        (status = UNAUTHORIZED, body = Problem),
        (status = FORBIDDEN, description = "Superusers cannot delete themselves", body = Problem),
    )
)]
async fn delete_me(
    State(state): State<AppState>,
    CurrentUser(user): CurrentUser,
) -> AppResult<Json<Message>> {
    // Deleting the last administrator would leave the system with no way back in, and there
    // is no way to tell from here whether another one exists that is not a race.
    if user.is_superuser {
        return Err(AppError::Forbidden);
    }

    services::users::delete(state.db(), user.id).await?;

    Ok(Json(Message::new("account deleted")))
}

/// Read a user
///
/// Anyone may read themselves; only superusers may read anyone else.
#[utoipa::path(
    get,
    path = "/{id}",
    tag = "users",
    params(("id" = Uuid, Path, description = "The user to read")),
    security(("bearer" = [])),
    responses(
        (status = OK, body = UserPublic),
        (status = UNAUTHORIZED, body = Problem),
        (status = FORBIDDEN, description = "Reading someone else without being a superuser", body = Problem),
        (status = NOT_FOUND, body = Problem),
    )
)]
async fn read(
    State(state): State<AppState>,
    CurrentUser(user): CurrentUser,
    Path(id): Path<Uuid>,
) -> AppResult<Json<UserPublic>> {
    if id != user.id && !user.is_superuser {
        return Err(AppError::Forbidden);
    }

    let found = repo::users::find_by_id(state.db(), id)
        .await?
        .ok_or(AppError::NotFound)?;

    Ok(Json(found.into()))
}

/// Update a user
///
/// Including their password and privileges. Superusers only. Setting a password ends every
/// session that user has.
#[utoipa::path(
    patch,
    path = "/{id}",
    tag = "users",
    params(("id" = Uuid, Path, description = "The user to update")),
    request_body = UserUpdate,
    security(("bearer" = [])),
    responses(
        (status = OK, body = UserPublic),
        (status = UNAUTHORIZED, body = Problem),
        (status = FORBIDDEN, description = "Not a superuser", body = Problem),
        (status = NOT_FOUND, body = Problem),
        (status = CONFLICT, description = "The address is taken", body = Problem),
        (status = UNPROCESSABLE_ENTITY, body = Problem),
    )
)]
async fn update(
    State(state): State<AppState>,
    Superuser(_admin): Superuser,
    Path(id): Path<Uuid>,
    ValidatedJson(payload): ValidatedJson<UserUpdate>,
) -> AppResult<Json<UserPublic>> {
    let updated = services::users::update(state.db(), id, payload).await?;

    Ok(Json(updated.into()))
}

/// Delete a user
///
/// Superusers only, and never yourself. Their items and sessions go with them.
#[utoipa::path(
    delete,
    path = "/{id}",
    tag = "users",
    params(("id" = Uuid, Path, description = "The user to delete")),
    security(("bearer" = [])),
    responses(
        (status = OK, body = Message),
        (status = UNAUTHORIZED, body = Problem),
        (status = FORBIDDEN, description = "Not a superuser, or deleting yourself", body = Problem),
        (status = NOT_FOUND, body = Problem),
    )
)]
async fn delete(
    State(state): State<AppState>,
    Superuser(current): Superuser,
    Path(id): Path<Uuid>,
) -> AppResult<Json<Message>> {
    if id == current.id {
        return Err(AppError::Forbidden);
    }

    services::users::delete(state.db(), id).await?;

    Ok(Json(Message::new("user deleted")))
}
