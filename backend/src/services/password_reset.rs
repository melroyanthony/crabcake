use time::{Duration, OffsetDateTime};

use crate::{
    AppError, AppResult, AppState,
    auth::{password, token},
    email, jobs,
    models::Password,
    repo,
};

/// Starts a reset by emailing a link.
///
/// Says nothing about whether the address is registered, and does the same work either way, so
/// that this endpoint cannot be used to discover who has an account. The caller always sees the
/// same answer.
pub async fn request(state: &AppState, address: &str) -> AppResult<()> {
    let Some(user) = repo::users::find_by_email(state.db(), address).await? else {
        tracing::info!("password reset requested for an unknown address");
        return Ok(());
    };

    if !user.is_active {
        tracing::info!(user = %user.id, "password reset requested for a deactivated account");
        return Ok(());
    }

    // Asking again makes any earlier link useless, so a forwarded or leaked email stops working
    // as soon as the real owner asks for another.
    repo::password_reset_tokens::invalidate_all_for_user(state.db(), user.id).await?;

    let reset = token::generate();
    let expires_at = OffsetDateTime::now_utc()
        + Duration::hours(state.config().password_reset_token_expire_hours);

    repo::password_reset_tokens::insert(state.db(), user.id, &reset.digest, expires_at).await?;

    let message = email::templates::reset_password(state.config(), &user.email, &reset.plaintext)?;
    jobs::enqueue(state.emails(), message).await?;

    Ok(())
}

/// Finishes a reset.
///
/// The token is claimed and the password replaced, then every session that user had is ended:
/// whoever prompted the reset may well have been signed in already.
pub async fn confirm(state: &AppState, plaintext: &str, new_password: &Password) -> AppResult<()> {
    let claimed = repo::password_reset_tokens::claim(state.db(), &token::digest(plaintext))
        .await?
        .ok_or_else(|| AppError::validation("that reset link is invalid or has expired"))?;

    let hashed = password::hash(new_password.expose())?;
    repo::users::update_password(state.db(), claimed.user_id, &hashed).await?;
    repo::refresh_tokens::revoke_all_for_user(state.db(), claimed.user_id).await?;

    tracing::info!(user = %claimed.user_id, "password reset");
    Ok(())
}
