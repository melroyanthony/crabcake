use sqlx::PgPool;
use time::OffsetDateTime;
use uuid::Uuid;

use crate::AppResult;

pub struct StoredResetToken {
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
        insert into password_reset_tokens (user_id, token_hash, expires_at)
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

/// Claims a token, returning who it belongs to. The update is the check: marking it used and
/// finding it unused happen in one statement, so two requests racing with the same link cannot
/// both succeed.
pub async fn claim(pool: &PgPool, digest: &str) -> AppResult<Option<StoredResetToken>> {
    let claimed = sqlx::query_as!(
        StoredResetToken,
        r#"
        update password_reset_tokens
        set used_at = now()
        where token_hash = $1
          and used_at is null
          and expires_at > now()
        returning id, user_id
        "#,
        digest
    )
    .fetch_optional(pool)
    .await?;

    Ok(claimed)
}

/// Invalidates any outstanding links for a user, so that asking for a second email makes the
/// first one useless.
pub async fn invalidate_all_for_user(pool: &PgPool, user_id: Uuid) -> AppResult<u64> {
    let result = sqlx::query!(
        "update password_reset_tokens set used_at = now() where user_id = $1 and used_at is null",
        user_id
    )
    .execute(pool)
    .await?;

    Ok(result.rows_affected())
}
