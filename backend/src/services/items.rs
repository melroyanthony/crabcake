use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    AppError, AppResult,
    models::{Item, ItemCreate, ItemUpdate, Page, Pagination, User},
    repo,
};

/// Superusers see every item; everyone else sees their own.
fn visible_owner(user: &User) -> Option<Uuid> {
    if user.is_superuser {
        None
    } else {
        Some(user.id)
    }
}

pub async fn list(pool: &PgPool, user: &User, page: &Pagination) -> AppResult<Page<Item>> {
    let owner = visible_owner(user);

    let items = repo::items::list(pool, owner, page.skip, page.limit).await?;
    let count = repo::items::count(pool, owner).await?;

    Ok(Page::new(items, count))
}

/// Fetches an item the user is allowed to see. Someone else's item reads as missing rather
/// than forbidden, so that guessing identifiers cannot confirm which ones exist.
pub async fn get(pool: &PgPool, user: &User, id: Uuid) -> AppResult<Item> {
    let item = repo::items::find(pool, id)
        .await?
        .ok_or(AppError::NotFound)?;

    if item.owner_id != user.id && !user.is_superuser {
        return Err(AppError::NotFound);
    }

    Ok(item)
}

pub async fn create(pool: &PgPool, user: &User, payload: ItemCreate) -> AppResult<Item> {
    repo::items::create(
        pool,
        user.id,
        &payload.title,
        payload.description.as_deref(),
    )
    .await
}

pub async fn update(pool: &PgPool, user: &User, id: Uuid, payload: ItemUpdate) -> AppResult<Item> {
    get(pool, user, id).await?;

    repo::items::update(
        pool,
        id,
        payload.title.as_deref(),
        payload.description.as_deref(),
    )
    .await
}

pub async fn delete(pool: &PgPool, user: &User, id: Uuid) -> AppResult<()> {
    get(pool, user, id).await?;

    repo::items::delete(pool, id).await
}
