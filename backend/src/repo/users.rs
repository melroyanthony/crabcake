use sqlx::PgPool;
use uuid::Uuid;

use crate::{AppError, AppResult, models::User};

/// The fields needed to insert a user. A struct rather than five positional arguments,
/// because `is_active` and `is_superuser` sit next to each other and are both `bool`.
#[derive(Debug)]
pub struct NewUser<'a> {
    pub email: &'a str,
    pub hashed_password: &'a str,
    pub full_name: Option<&'a str>,
    pub is_active: bool,
    pub is_superuser: bool,
}

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

pub async fn list(pool: &PgPool, skip: i64, limit: i64) -> AppResult<Vec<User>> {
    let users = sqlx::query_as!(
        User,
        r#"
        select id, email, hashed_password, full_name, is_active, is_superuser,
               created_at, updated_at
        from users
        order by created_at desc
        offset $1
        limit $2
        "#,
        skip,
        limit
    )
    .fetch_all(pool)
    .await?;

    Ok(users)
}

pub async fn count(pool: &PgPool) -> AppResult<i64> {
    let count = sqlx::query_scalar!(r#"select count(*) as "count!" from users"#)
        .fetch_one(pool)
        .await?;

    Ok(count)
}

pub async fn create(pool: &PgPool, user: NewUser<'_>) -> AppResult<User> {
    let created = sqlx::query_as!(
        User,
        r#"
        insert into users (email, hashed_password, full_name, is_active, is_superuser)
        values ($1, $2, $3, $4, $5)
        returning id, email, hashed_password, full_name, is_active, is_superuser,
                  created_at, updated_at
        "#,
        user.email,
        user.hashed_password,
        user.full_name,
        user.is_active,
        user.is_superuser
    )
    .fetch_one(pool)
    .await?;

    Ok(created)
}

/// Applies a partial update. A `None` field is left as it was, so a client can send only what
/// changed; clearing a name therefore needs an empty string rather than null.
pub async fn update(
    pool: &PgPool,
    id: Uuid,
    email: Option<&str>,
    full_name: Option<&str>,
    is_active: Option<bool>,
    is_superuser: Option<bool>,
) -> AppResult<User> {
    let user = sqlx::query_as!(
        User,
        r#"
        update users
        set email = coalesce($2, email),
            full_name = coalesce($3, full_name),
            is_active = coalesce($4, is_active),
            is_superuser = coalesce($5, is_superuser),
            updated_at = now()
        where id = $1
        returning id, email, hashed_password, full_name, is_active, is_superuser,
                  created_at, updated_at
        "#,
        id,
        email,
        full_name,
        is_active,
        is_superuser
    )
    .fetch_optional(pool)
    .await?
    .ok_or(AppError::NotFound)?;

    Ok(user)
}

pub async fn update_password(pool: &PgPool, id: Uuid, hashed_password: &str) -> AppResult<()> {
    let result = sqlx::query!(
        "update users set hashed_password = $2, updated_at = now() where id = $1",
        id,
        hashed_password
    )
    .execute(pool)
    .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound);
    }

    Ok(())
}

/// Items and refresh tokens are removed by `on delete cascade`, so deleting a user really
/// does leave nothing behind.
pub async fn delete(pool: &PgPool, id: Uuid) -> AppResult<()> {
    let result = sqlx::query!("delete from users where id = $1", id)
        .execute(pool)
        .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound);
    }

    Ok(())
}
