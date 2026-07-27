use axum::{
    Json, Router,
    extract::{Path, State},
    http::StatusCode,
    routing::get,
};
use uuid::Uuid;

use crate::{
    AppResult, AppState,
    api::extract::{ValidatedJson, ValidatedQuery},
    auth::CurrentUser,
    models::{Item, ItemCreate, ItemUpdate, Message, Page, Pagination},
    services,
};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(list).post(create))
        .route("/{id}", get(read).patch(update).delete(delete))
}

/// Lists the caller's items, or every item when the caller is a superuser.
async fn list(
    State(state): State<AppState>,
    CurrentUser(user): CurrentUser,
    ValidatedQuery(page): ValidatedQuery<Pagination>,
) -> AppResult<Json<Page<Item>>> {
    let items = services::items::list(state.db(), &user, &page).await?;

    Ok(Json(items))
}

async fn create(
    State(state): State<AppState>,
    CurrentUser(user): CurrentUser,
    ValidatedJson(payload): ValidatedJson<ItemCreate>,
) -> AppResult<(StatusCode, Json<Item>)> {
    let item = services::items::create(state.db(), &user, payload).await?;

    Ok((StatusCode::CREATED, Json(item)))
}

async fn read(
    State(state): State<AppState>,
    CurrentUser(user): CurrentUser,
    Path(id): Path<Uuid>,
) -> AppResult<Json<Item>> {
    let item = services::items::get(state.db(), &user, id).await?;

    Ok(Json(item))
}

async fn update(
    State(state): State<AppState>,
    CurrentUser(user): CurrentUser,
    Path(id): Path<Uuid>,
    ValidatedJson(payload): ValidatedJson<ItemUpdate>,
) -> AppResult<Json<Item>> {
    let item = services::items::update(state.db(), &user, id, payload).await?;

    Ok(Json(item))
}

async fn delete(
    State(state): State<AppState>,
    CurrentUser(user): CurrentUser,
    Path(id): Path<Uuid>,
) -> AppResult<Json<Message>> {
    services::items::delete(state.db(), &user, id).await?;

    Ok(Json(Message::new("item deleted")))
}
