use sqlx::PgPool;
use uuid::Uuid;

use crate::{AppError, AppResult, models::Item};

/// Lists items newest first. `owner` of `None` means every item, which is how the admin view
/// is served; passing an owner scopes it to that user.
pub async fn list(
    pool: &PgPool,
    owner: Option<Uuid>,
    skip: i64,
    limit: i64,
) -> AppResult<Vec<Item>> {
    let items = sqlx::query_as!(
        Item,
        r#"
        select id, owner_id, title, description, created_at, updated_at
        from items
        where $1::uuid is null or owner_id = $1
        order by created_at desc
        offset $2
        limit $3
        "#,
        owner,
        skip,
        limit
    )
    .fetch_all(pool)
    .await?;

    Ok(items)
}

pub async fn count(pool: &PgPool, owner: Option<Uuid>) -> AppResult<i64> {
    let count = sqlx::query_scalar!(
        r#"select count(*) as "count!" from items where $1::uuid is null or owner_id = $1"#,
        owner
    )
    .fetch_one(pool)
    .await?;

    Ok(count)
}

pub async fn find(pool: &PgPool, id: Uuid) -> AppResult<Option<Item>> {
    let item = sqlx::query_as!(
        Item,
        r#"
        select id, owner_id, title, description, created_at, updated_at
        from items
        where id = $1
        "#,
        id
    )
    .fetch_optional(pool)
    .await?;

    Ok(item)
}

pub async fn create(
    pool: &PgPool,
    owner_id: Uuid,
    title: &str,
    description: Option<&str>,
) -> AppResult<Item> {
    let item = sqlx::query_as!(
        Item,
        r#"
        insert into items (owner_id, title, description)
        values ($1, $2, $3)
        returning id, owner_id, title, description, created_at, updated_at
        "#,
        owner_id,
        title,
        description
    )
    .fetch_one(pool)
    .await?;

    Ok(item)
}

/// Applies a partial update. A `None` field is left as it was, so a client can send only what
/// changed. Clearing a description therefore needs an empty string rather than null.
pub async fn update(
    pool: &PgPool,
    id: Uuid,
    title: Option<&str>,
    description: Option<&str>,
) -> AppResult<Item> {
    let item = sqlx::query_as!(
        Item,
        r#"
        update items
        set title = coalesce($2, title),
            description = coalesce($3, description),
            updated_at = now()
        where id = $1
        returning id, owner_id, title, description, created_at, updated_at
        "#,
        id,
        title,
        description
    )
    .fetch_optional(pool)
    .await?
    .ok_or(AppError::NotFound)?;

    Ok(item)
}

pub async fn delete(pool: &PgPool, id: Uuid) -> AppResult<()> {
    let result = sqlx::query!("delete from items where id = $1", id)
        .execute(pool)
        .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound);
    }

    Ok(())
}
