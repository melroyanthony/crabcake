//! Items over HTTP: ownership, listing scope, and the way a non-owner is answered with a 404
//! rather than a 403, so the existence of somebody else's item is not confirmed.

mod common;

use axum::http::{Method, StatusCode};
use common::{app, create_user, http_status, json, login, path};
use sqlx::PgPool;
use uuid::Uuid;

async fn create_item(app: &axum::Router, token: &str, title: &str) -> Uuid {
    let (code, body) = json::<serde_json::Value>(
        app,
        Method::POST,
        "/api/v1/items",
        Some(token),
        Some(&serde_json::json!({ "title": title })),
    )
    .await;

    assert_eq!(code, StatusCode::CREATED);
    body["id"].as_str().unwrap().parse().unwrap()
}

#[sqlx::test]
async fn creating_an_item_needs_to_be_signed_in(pool: PgPool) {
    let app = app(common::state(&pool).await);

    let code = http_status(
        &app,
        Method::POST,
        "/api/v1/items",
        None,
        Some(&serde_json::json!({ "title": "Buy oat milk" })),
    )
    .await;

    assert_eq!(code, StatusCode::UNAUTHORIZED);
}

#[sqlx::test]
async fn a_user_sees_only_their_own_items(pool: PgPool) {
    let app = app(common::state(&pool).await);
    create_user(&pool, "ada@example.com", "correct-horse", false).await;
    create_user(&pool, "grace@example.com", "correct-horse", false).await;
    let ada = login(&app, "ada@example.com", "correct-horse").await;
    let grace = login(&app, "grace@example.com", "correct-horse").await;

    create_item(&app, &ada.access_token, "Ada's list").await;
    create_item(&app, &grace.access_token, "Grace's list").await;

    let (code, body) = json::<serde_json::Value>(
        &app,
        Method::GET,
        "/api/v1/items",
        Some(&ada.access_token),
        None::<&serde_json::Value>,
    )
    .await;

    assert_eq!(code, StatusCode::OK);
    assert_eq!(body["count"], 1);
    assert_eq!(body["data"][0]["title"], "Ada's list");
}

#[sqlx::test]
async fn a_superuser_sees_everyones_items(pool: PgPool) {
    let app = app(common::state(&pool).await);
    create_user(&pool, "ada@example.com", "correct-horse", false).await;
    create_user(&pool, "admin@example.com", "correct-horse", true).await;
    let ada = login(&app, "ada@example.com", "correct-horse").await;
    let admin = login(&app, "admin@example.com", "correct-horse").await;

    create_item(&app, &ada.access_token, "Ada's list").await;
    create_item(&app, &admin.access_token, "Admin's list").await;

    let (code, body) = json::<serde_json::Value>(
        &app,
        Method::GET,
        "/api/v1/items",
        Some(&admin.access_token),
        None::<&serde_json::Value>,
    )
    .await;

    assert_eq!(code, StatusCode::OK);
    assert_eq!(body["count"], 2);
}

#[sqlx::test]
async fn a_non_owner_gets_a_404_not_a_403(pool: PgPool) {
    let app = app(common::state(&pool).await);
    create_user(&pool, "ada@example.com", "correct-horse", false).await;
    create_user(&pool, "grace@example.com", "correct-horse", false).await;
    let ada = login(&app, "ada@example.com", "correct-horse").await;
    let grace = login(&app, "grace@example.com", "correct-horse").await;
    let id = create_item(&app, &ada.access_token, "Ada's list").await;

    for method in [Method::GET, Method::PATCH, Method::DELETE] {
        let body = if method == Method::PATCH {
            Some(serde_json::json!({ "title": "stolen" }))
        } else {
            None
        };

        let code = http_status(
            &app,
            method.clone(),
            &path("/api/v1/items/{id}", id),
            Some(&grace.access_token),
            body.as_ref(),
        )
        .await;

        assert_eq!(
            code,
            StatusCode::NOT_FOUND,
            "{method} by a non-owner must not confirm the item exists"
        );
    }
}

#[sqlx::test]
async fn the_owner_can_update_and_delete(pool: PgPool) {
    let app = app(common::state(&pool).await);
    create_user(&pool, "ada@example.com", "correct-horse", false).await;
    let ada = login(&app, "ada@example.com", "correct-horse").await;
    let id = create_item(&app, &ada.access_token, "Buy oat milk").await;

    let (updated_code, updated) = json::<serde_json::Value>(
        &app,
        Method::PATCH,
        &path("/api/v1/items/{id}", id),
        Some(&ada.access_token),
        Some(&serde_json::json!({ "title": "Buy oat milk and eggs" })),
    )
    .await;

    assert_eq!(updated_code, StatusCode::OK);
    assert_eq!(updated["title"], "Buy oat milk and eggs");

    let deleted = http_status(
        &app,
        Method::DELETE,
        &path("/api/v1/items/{id}", id),
        Some(&ada.access_token),
        None::<&serde_json::Value>,
    )
    .await;

    assert_eq!(deleted, StatusCode::OK);

    let missing = http_status(
        &app,
        Method::GET,
        &path("/api/v1/items/{id}", id),
        Some(&ada.access_token),
        None::<&serde_json::Value>,
    )
    .await;

    assert_eq!(missing, StatusCode::NOT_FOUND);
}

#[sqlx::test]
async fn a_superuser_can_edit_someone_elses_item(pool: PgPool) {
    let app = app(common::state(&pool).await);
    let ada_user = create_user(&pool, "ada@example.com", "correct-horse", false).await;
    create_user(&pool, "admin@example.com", "correct-horse", true).await;
    let ada = login(&app, "ada@example.com", "correct-horse").await;
    let admin = login(&app, "admin@example.com", "correct-horse").await;
    let id = create_item(&app, &ada.access_token, "Ada's list").await;

    let (code, body) = json::<serde_json::Value>(
        &app,
        Method::PATCH,
        &path("/api/v1/items/{id}", id),
        Some(&admin.access_token),
        Some(&serde_json::json!({ "title": "Reviewed by admin" })),
    )
    .await;

    assert_eq!(code, StatusCode::OK);
    assert_eq!(body["title"], "Reviewed by admin");
    // Ownership does not change hands when an administrator edits.
    assert_eq!(body["owner_id"], ada_user.id.to_string());
}

#[sqlx::test]
async fn an_empty_title_is_rejected(pool: PgPool) {
    let app = app(common::state(&pool).await);
    create_user(&pool, "ada@example.com", "correct-horse", false).await;
    let ada = login(&app, "ada@example.com", "correct-horse").await;

    let code = http_status(
        &app,
        Method::POST,
        "/api/v1/items",
        Some(&ada.access_token),
        Some(&serde_json::json!({ "title": "" })),
    )
    .await;

    assert_eq!(code, StatusCode::UNPROCESSABLE_ENTITY);
}
