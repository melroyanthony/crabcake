use axum::{
    extract::FromRequestParts,
    http::{header::AUTHORIZATION, request::Parts},
};

use crate::{AppError, AppState, auth::jwt, models::User, repo};

/// An authenticated, active user. Any handler that takes this argument is closed to anonymous
/// callers, which is checked by the compiler rather than by remembering to call a guard.
#[derive(Debug, Clone)]
pub struct CurrentUser(pub User);

impl FromRequestParts<AppState> for CurrentUser {
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let token = bearer_token(parts).ok_or(AppError::Unauthorized)?;
        let claims = jwt::decode_access_token(state.config(), token)?;

        let user = repo::users::find_by_id(state.db(), claims.sub)
            .await?
            .ok_or(AppError::Unauthorized)?;

        // A token outlives a deactivation, so activity is checked on every request rather
        // than only at sign-in.
        if !user.is_active {
            return Err(AppError::Forbidden);
        }

        Ok(Self(user))
    }
}

/// An authenticated user who is also a superuser.
#[derive(Debug, Clone)]
pub struct Superuser(pub User);

impl FromRequestParts<AppState> for Superuser {
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let CurrentUser(user) = CurrentUser::from_request_parts(parts, state).await?;

        if user.is_superuser {
            Ok(Self(user))
        } else {
            Err(AppError::Forbidden)
        }
    }
}

fn bearer_token(parts: &Parts) -> Option<&str> {
    parts
        .headers
        .get(AUTHORIZATION)?
        .to_str()
        .ok()?
        .strip_prefix("Bearer ")
        .map(str::trim)
        .filter(|token| !token.is_empty())
}
