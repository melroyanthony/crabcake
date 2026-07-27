use sqlx::PgPool;
use uuid::Uuid;

use crate::{AppResult, models::User};

pub async fn find_by_id(pool: &PgPool, id: Uuid) -> AppResult<Option<User>> {
    let user = sqlx::query_as!(
        User,
        r#"
        select id, email, hashed_password, full_name, is_active, is_superuser,
               created_at, updated_at
        from users
        where id = $1
        "#,
        id
    )
    .fetch_optional(pool)
    .await?;

    Ok(user)
}

/// Looks a user up by address, case-insensitively, matching the unique index on
/// `lower(email)`.
pub async fn find_by_email(pool: &PgPool, email: &str) -> AppResult<Option<User>> {
    let user = sqlx::query_as!(
        User,
        r#"
        select id, email, hashed_password, full_name, is_active, is_superuser,
               created_at, updated_at
        from users
        where lower(email) = lower($1)
        "#,
        email
    )
    .fetch_optional(pool)
    .await?;

    Ok(user)
}

pub async fn create(
    pool: &PgPool,
    email: &str,
    hashed_password: &str,
    full_name: Option<&str>,
    is_superuser: bool,
) -> AppResult<User> {
    let user = sqlx::query_as!(
        User,
        r#"
        insert into users (email, hashed_password, full_name, is_superuser)
        values ($1, $2, $3, $4)
        returning id, email, hashed_password, full_name, is_active, is_superuser,
                  created_at, updated_at
        "#,
        email,
        hashed_password,
        full_name,
        is_superuser
    )
    .fetch_one(pool)
    .await?;

    Ok(user)
}
