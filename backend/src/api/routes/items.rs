use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use utoipa_axum::{router::OpenApiRouter, routes};
use uuid::Uuid;

use crate::{
    AppResult, AppState,
    api::extract::{ValidatedJson, ValidatedQuery},
    auth::CurrentUser,
    error::Problem,
    models::{Item, ItemCreate, ItemUpdate, Message, Page, Pagination},
    services,
};

pub fn router() -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        .routes(routes!(list, create))
        .routes(routes!(read, update, delete))
}

/// List items
///
/// Your own items, or every item when you are a superuser.
#[utoipa::path(
    get,
    path = "/",
    tag = "items",
    params(Pagination),
    security(("bearer" = [])),
    responses(
        (status = OK, body = Page<Item>),
        (status = UNAUTHORIZED, body = Problem),
        (status = UNPROCESSABLE_ENTITY, description = "Invalid paging", body = Problem),
    )
)]
async fn list(
    State(state): State<AppState>,
    CurrentUser(user): CurrentUser,
    ValidatedQuery(page): ValidatedQuery<Pagination>,
) -> AppResult<Json<Page<Item>>> {
    let items = services::items::list(state.db(), &user, &page).await?;

    Ok(Json(items))
}

/// Create an item
///
/// Owned by whoever creates it.
#[utoipa::path(
    post,
    path = "/",
    tag = "items",
    request_body = ItemCreate,
    security(("bearer" = [])),
    responses(
        (status = CREATED, body = Item),
        (status = UNAUTHORIZED, body = Problem),
        (status = UNPROCESSABLE_ENTITY, body = Problem),
    )
)]
async fn create(
    State(state): State<AppState>,
    CurrentUser(user): CurrentUser,
    ValidatedJson(payload): ValidatedJson<ItemCreate>,
) -> AppResult<(StatusCode, Json<Item>)> {
    let item = services::items::create(state.db(), &user, payload).await?;

    Ok((StatusCode::CREATED, Json(item)))
}

/// Read an item
///
/// Someone else's item reads as missing rather than forbidden, so that guessing identifiers
/// cannot confirm which ones exist.
#[utoipa::path(
    get,
    path = "/{id}",
    tag = "items",
    params(("id" = Uuid, Path, description = "The item to read")),
    security(("bearer" = [])),
    responses(
        (status = OK, body = Item),
        (status = UNAUTHORIZED, body = Problem),
        (status = NOT_FOUND, description = "No such item, or not yours", body = Problem),
    )
)]
async fn read(
    State(state): State<AppState>,
    CurrentUser(user): CurrentUser,
    Path(id): Path<Uuid>,
) -> AppResult<Json<Item>> {
    let item = services::items::get(state.db(), &user, id).await?;

    Ok(Json(item))
}

/// Update an item
///
/// Every field is optional; anything left out keeps its current value.
#[utoipa::path(
    patch,
    path = "/{id}",
    tag = "items",
    params(("id" = Uuid, Path, description = "The item to update")),
    request_body = ItemUpdate,
    security(("bearer" = [])),
    responses(
        (status = OK, body = Item),
        (status = UNAUTHORIZED, body = Problem),
        (status = NOT_FOUND, description = "No such item, or not yours", body = Problem),
        (status = UNPROCESSABLE_ENTITY, body = Problem),
    )
)]
async fn update(
    State(state): State<AppState>,
    CurrentUser(user): CurrentUser,
    Path(id): Path<Uuid>,
    ValidatedJson(payload): ValidatedJson<ItemUpdate>,
) -> AppResult<Json<Item>> {
    let item = services::items::update(state.db(), &user, id, payload).await?;

    Ok(Json(item))
}

/// Delete an item
#[utoipa::path(
    delete,
    path = "/{id}",
    tag = "items",
    params(("id" = Uuid, Path, description = "The item to delete")),
    security(("bearer" = [])),
    responses(
        (status = OK, body = Message),
        (status = UNAUTHORIZED, body = Problem),
        (status = NOT_FOUND, description = "No such item, or not yours", body = Problem),
    )
)]
async fn delete(
    State(state): State<AppState>,
    CurrentUser(user): CurrentUser,
    Path(id): Path<Uuid>,
) -> AppResult<Json<Message>> {
    services::items::delete(state.db(), &user, id).await?;

    Ok(Json(Message::new("item deleted")))
}
