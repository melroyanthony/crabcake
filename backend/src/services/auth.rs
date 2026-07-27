use serde::Serialize;
use time::{Duration, OffsetDateTime};
use uuid::Uuid;

use crate::{
    AppError, AppResult, AppState,
    auth::{jwt, password, refresh},
    models::User,
    repo,
};

/// A hash of nothing in particular. Verifying against it when no user matches keeps the
/// response time of a wrong address indistinguishable from a wrong password, so the endpoint
/// cannot be used to discover which addresses are registered.
const DUMMY_HASH: &str = "$argon2id$v=19$m=19456,t=2,p=1$c2FsdHNhbHRzYWx0c2E$\
                          Kq5ETPFCFVgUqQZ0F2QJZ7FQE0zVJqQvVYnQZQe0m1A";

#[derive(Debug, Serialize)]
pub struct TokenPair {
    pub access_token: String,
    pub refresh_token: String,
    pub token_type: &'static str,
}

pub async fn login(state: &AppState, email: &str, plaintext: &str) -> AppResult<TokenPair> {
    let user = repo::users::find_by_email(state.db(), email).await?;

    let Some(user) = user else {
        password::verify(plaintext, DUMMY_HASH);
        return Err(AppError::Unauthorized);
    };

    if !password::verify(plaintext, &user.hashed_password) {
        return Err(AppError::Unauthorized);
    }

    if !user.is_active {
        return Err(AppError::Forbidden);
    }

    issue_pair(state, &user).await
}

/// Exchanges a refresh token for a new pair, revoking the old one. Rotating on every use
/// means a stolen token is good for at most one refresh, and the theft shows up as the
/// legitimate client being signed out.
pub async fn refresh(state: &AppState, plaintext: &str) -> AppResult<TokenPair> {
    let digest = refresh::digest(plaintext);

    let stored = repo::refresh_tokens::find_active(state.db(), &digest)
        .await?
        .ok_or(AppError::Unauthorized)?;

    let user = repo::users::find_by_id(state.db(), stored.user_id)
        .await?
        .ok_or(AppError::Unauthorized)?;

    if !user.is_active {
        return Err(AppError::Forbidden);
    }

    repo::refresh_tokens::revoke(state.db(), stored.id).await?;

    issue_pair(state, &user).await
}

/// Ends a single session. Unknown tokens succeed quietly: signing out should never fail, and
/// reporting which tokens exist would leak information.
pub async fn logout(state: &AppState, plaintext: &str) -> AppResult<()> {
    repo::refresh_tokens::revoke_by_digest(state.db(), &refresh::digest(plaintext)).await
}

/// Ends every session a user has.
pub async fn logout_everywhere(state: &AppState, user_id: Uuid) -> AppResult<u64> {
    repo::refresh_tokens::revoke_all_for_user(state.db(), user_id).await
}

async fn issue_pair(state: &AppState, user: &User) -> AppResult<TokenPair> {
    let access_token = jwt::issue_access_token(state.config(), user.id)?;

    let token = refresh::generate();
    let expires_at =
        OffsetDateTime::now_utc() + Duration::days(state.config().refresh_token_expire_days);

    repo::refresh_tokens::insert(state.db(), user.id, &token.digest, expires_at).await?;

    Ok(TokenPair {
        access_token,
        refresh_token: token.plaintext,
        token_type: "bearer",
    })
}
