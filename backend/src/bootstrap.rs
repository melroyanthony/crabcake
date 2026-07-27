use secrecy::ExposeSecret;
use sqlx::PgPool;

use crate::{AppResult, Config, auth::password, repo};

/// Creates the first superuser if it does not exist yet, so a fresh database is usable
/// immediately. Idempotent, because it runs on every startup.
pub async fn ensure_first_superuser(pool: &PgPool, config: &Config) -> AppResult<()> {
    if repo::users::find_by_email(pool, &config.first_superuser)
        .await?
        .is_some()
    {
        return Ok(());
    }

    let hashed = password::hash(config.first_superuser_password.expose_secret())?;
    repo::users::create(pool, &config.first_superuser, &hashed, None, true).await?;

    tracing::info!(email = %config.first_superuser, "created the first superuser");
    Ok(())
}
