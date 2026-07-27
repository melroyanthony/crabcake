use axum::{
    Json, Router,
    extract::{Path, State},
    http::StatusCode,
    routing::{get, patch, post},
};
use uuid::Uuid;

use crate::{
    AppError, AppResult, AppState,
    api::extract::{ValidatedJson, ValidatedQuery},
    auth::{CurrentUser, Superuser},
    models::{
        Message, Page, Pagination, PasswordUpdate, UserCreate, UserPublic, UserRegister,
        UserUpdate, UserUpdateMe,
    },
    repo, services,
};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(list).post(create))
        .route("/signup", post(signup))
        .route("/me", get(read_me).patch(update_me).delete(delete_me))
        .route("/me/password", patch(update_my_password))
        .route("/{id}", get(read).patch(update).delete(delete))
}

/// Lists every user. Superusers only.
async fn list(
    State(state): State<AppState>,
    Superuser(_): Superuser,
    ValidatedQuery(page): ValidatedQuery<Pagination>,
) -> AppResult<Json<Page<UserPublic>>> {
    let users = repo::users::list(state.db(), page.skip, page.limit).await?;
    let count = repo::users::count(state.db()).await?;

    Ok(Json(Page::new(
        users.into_iter().map(UserPublic::from).collect(),
        count,
    )))
}

/// Creates a user with any combination of fields. Superusers only.
async fn create(
    State(state): State<AppState>,
    Superuser(_): Superuser,
    ValidatedJson(payload): ValidatedJson<UserCreate>,
) -> AppResult<(StatusCode, Json<UserPublic>)> {
    let user = services::users::create(state.db(), payload).await?;

    Ok((StatusCode::CREATED, Json(user.into())))
}

/// Open registration, for anyone.
async fn signup(
    State(state): State<AppState>,
    ValidatedJson(payload): ValidatedJson<UserRegister>,
) -> AppResult<(StatusCode, Json<UserPublic>)> {
    let user = services::users::register(state.db(), payload).await?;

    Ok((StatusCode::CREATED, Json(user.into())))
}

async fn read_me(CurrentUser(user): CurrentUser) -> Json<UserPublic> {
    Json(user.into())
}

async fn update_me(
    State(state): State<AppState>,
    CurrentUser(user): CurrentUser,
    ValidatedJson(payload): ValidatedJson<UserUpdateMe>,
) -> AppResult<Json<UserPublic>> {
    let updated = services::users::update_me(state.db(), user.id, payload).await?;

    Ok(Json(updated.into()))
}

async fn update_my_password(
    State(state): State<AppState>,
    CurrentUser(user): CurrentUser,
    ValidatedJson(payload): ValidatedJson<PasswordUpdate>,
) -> AppResult<Json<Message>> {
    services::users::change_password(state.db(), &user, payload).await?;

    Ok(Json(Message::new("password updated")))
}

/// Closes the caller's own account.
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

/// Reads any user as a superuser, or yourself as anyone.
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

/// Updates any user, including privileges. Superusers only.
async fn update(
    State(state): State<AppState>,
    Superuser(_): Superuser,
    Path(id): Path<Uuid>,
    ValidatedJson(payload): ValidatedJson<UserUpdate>,
) -> AppResult<Json<UserPublic>> {
    let updated = services::users::update(state.db(), id, payload).await?;

    Ok(Json(updated.into()))
}

/// Deletes any user but yourself. Superusers only.
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
