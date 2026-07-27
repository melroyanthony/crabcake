use sqlx::PgPool;
use time::OffsetDateTime;
use uuid::Uuid;

use crate::AppResult;

pub struct StoredRefreshToken {
    pub id: Uuid,
    pub user_id: Uuid,
}

pub async fn insert(
    pool: &PgPool,
    user_id: Uuid,
    digest: &str,
    expires_at: OffsetDateTime,
) -> AppResult<()> {
    sqlx::query!(
        r#"
        insert into refresh_tokens (user_id, token_hash, expires_at)
        values ($1, $2, $3)
        "#,
        user_id,
        digest,
        expires_at
    )
    .execute(pool)
    .await?;

    Ok(())
}

/// Finds a token that is neither expired nor revoked. Expiry is evaluated by the database so
/// that a skewed application clock cannot extend a session.
pub async fn find_active(pool: &PgPool, digest: &str) -> AppResult<Option<StoredRefreshToken>> {
    let token = sqlx::query_as!(
        StoredRefreshToken,
        r#"
        select id, user_id
        from refresh_tokens
        where token_hash = $1
          and revoked_at is null
          and expires_at > now()
        "#,
        digest
    )
    .fetch_optional(pool)
    .await?;

    Ok(token)
}

pub async fn revoke(pool: &PgPool, id: Uuid) -> AppResult<()> {
    sqlx::query!(
        "update refresh_tokens set revoked_at = now() where id = $1 and revoked_at is null",
        id
    )
    .execute(pool)
    .await?;

    Ok(())
}

/// Revokes every session a user has. Used when a refresh token is replayed, and available for
/// a "sign out everywhere" action.
pub async fn revoke_all_for_user(pool: &PgPool, user_id: Uuid) -> AppResult<u64> {
    let result = sqlx::query!(
        "update refresh_tokens set revoked_at = now() where user_id = $1 and revoked_at is null",
        user_id
    )
    .execute(pool)
    .await?;

    Ok(result.rows_affected())
}

/// Revokes by digest without needing to know the owner, for signing out a single session.
pub async fn revoke_by_digest(pool: &PgPool, digest: &str) -> AppResult<()> {
    sqlx::query!(
        "update refresh_tokens set revoked_at = now() where token_hash = $1 and revoked_at is null",
        digest
    )
    .execute(pool)
    .await?;

    Ok(())
}
