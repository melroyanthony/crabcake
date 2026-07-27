use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    AppError, AppResult,
    auth::password,
    error::OnUniqueViolation,
    models::{PasswordUpdate, User, UserCreate, UserRegister, UserUpdate, UserUpdateMe},
    repo::{self, users::NewUser},
};

const EMAIL_TAKEN: &str = "a user with that email address already exists";

/// Open registration. The resulting account is always an ordinary active user.
pub async fn register(pool: &PgPool, payload: UserRegister) -> AppResult<User> {
    let hashed_password = password::hash(payload.password.expose())?;

    repo::users::create(
        pool,
        NewUser {
            email: &payload.email,
            hashed_password: &hashed_password,
            full_name: payload.full_name.as_deref(),
            is_active: true,
            is_superuser: false,
        },
    )
    .await
    .on_unique_violation(EMAIL_TAKEN)
}

/// Creation by a superuser, which may set any field.
pub async fn create(pool: &PgPool, payload: UserCreate) -> AppResult<User> {
    let hashed_password = password::hash(payload.password.expose())?;

    repo::users::create(
        pool,
        NewUser {
            email: &payload.email,
            hashed_password: &hashed_password,
            full_name: payload.full_name.as_deref(),
            is_active: payload.is_active,
            is_superuser: payload.is_superuser,
        },
    )
    .await
    .on_unique_violation(EMAIL_TAKEN)
}

pub async fn update_me(pool: &PgPool, id: Uuid, payload: UserUpdateMe) -> AppResult<User> {
    repo::users::update(
        pool,
        id,
        payload.email.as_deref(),
        payload.full_name.as_deref(),
        None,
        None,
    )
    .await
    .on_unique_violation(EMAIL_TAKEN)
}

pub async fn update(pool: &PgPool, id: Uuid, payload: UserUpdate) -> AppResult<User> {
    if let Some(new_password) = &payload.password {
        let hashed_password = password::hash(new_password.expose())?;
        repo::users::update_password(pool, id, &hashed_password).await?;

        // An administrator resetting a password is either helping someone locked out or
        // shutting an intruder out. Both mean the existing sessions should stop working.
        repo::refresh_tokens::revoke_all_for_user(pool, id).await?;
    }

    repo::users::update(
        pool,
        id,
        payload.email.as_deref(),
        payload.full_name.as_deref(),
        payload.is_active,
        payload.is_superuser,
    )
    .await
    .on_unique_violation(EMAIL_TAKEN)
}

/// Changes a password after proving the current one. Every other session is ended, which is
/// what a user changing their password after a scare expects to happen.
pub async fn change_password(pool: &PgPool, user: &User, payload: PasswordUpdate) -> AppResult<()> {
    if !password::verify(payload.current_password.expose(), &user.hashed_password) {
        return Err(AppError::validation("current password is incorrect"));
    }

    if payload.current_password.expose() == payload.new_password.expose() {
        return Err(AppError::validation(
            "the new password must differ from the current one",
        ));
    }

    let hashed_password = password::hash(payload.new_password.expose())?;
    repo::users::update_password(pool, user.id, &hashed_password).await?;
    repo::refresh_tokens::revoke_all_for_user(pool, user.id).await?;

    Ok(())
}

pub async fn delete(pool: &PgPool, id: Uuid) -> AppResult<()> {
    repo::users::delete(pool, id).await
}
