//! Users over HTTP: open signup, self-service, and the privileges a superuser has that an
//! ordinary account does not.

mod common;

use axum::http::{Method, StatusCode};
use common::{app, create_user, http_status, json, login, path, problem};
use sqlx::PgPool;
use uuid::Uuid;

#[sqlx::test]
async fn signup_creates_an_ordinary_active_account(pool: PgPool) {
    let app = app(common::state(&pool).await);

    let (code, body) = json::<serde_json::Value>(
        &app,
        Method::POST,
        "/api/v1/users/signup",
        None,
        Some(&serde_json::json!({
            "email": "ada@example.com",
            "password": "correct-horse",
            "full_name": "Ada Lovelace",
            // Ignored: the register shape has no such fields, so this cannot self-promote.
            "is_superuser": true,
            "is_active": false,
        })),
    )
    .await;

    assert_eq!(code, StatusCode::CREATED);
    assert_eq!(body["email"], "ada@example.com");
    assert_eq!(body["full_name"], "Ada Lovelace");
    assert_eq!(body["is_active"], true);
    assert_eq!(body["is_superuser"], false);
}

#[sqlx::test]
async fn signup_refuses_a_taken_address(pool: PgPool) {
    let app = app(common::state(&pool).await);
    create_user(&pool, "ada@example.com", "correct-horse", false).await;

    let (code, body) = problem(
        &app,
        Method::POST,
        "/api/v1/users/signup",
        None,
        Some(&serde_json::json!({
            "email": "ada@example.com",
            "password": "a-different-password",
        })),
    )
    .await;

    assert_eq!(code, StatusCode::CONFLICT);
    assert_eq!(body["title"], "Conflict");
}

#[sqlx::test]
async fn me_returns_the_caller(pool: PgPool) {
    let app = app(common::state(&pool).await);
    let user = create_user(&pool, "ada@example.com", "correct-horse", false).await;
    let pair = login(&app, "ada@example.com", "correct-horse").await;

    let (code, body) = json::<serde_json::Value>(
        &app,
        Method::GET,
        "/api/v1/users/me",
        Some(&pair.access_token),
        None::<&serde_json::Value>,
    )
    .await;

    assert_eq!(code, StatusCode::OK);
    assert_eq!(body["id"], user.id.to_string());
}

#[sqlx::test]
async fn a_user_can_update_their_own_profile(pool: PgPool) {
    let app = app(common::state(&pool).await);
    create_user(&pool, "ada@example.com", "correct-horse", false).await;
    let pair = login(&app, "ada@example.com", "correct-horse").await;

    let (code, body) = json::<serde_json::Value>(
        &app,
        Method::PATCH,
        "/api/v1/users/me",
        Some(&pair.access_token),
        Some(&serde_json::json!({ "full_name": "Augusta Ada" })),
    )
    .await;

    assert_eq!(code, StatusCode::OK);
    assert_eq!(body["full_name"], "Augusta Ada");
}

#[sqlx::test]
async fn changing_password_ends_every_session(pool: PgPool) {
    let app = app(common::state(&pool).await);
    create_user(&pool, "ada@example.com", "correct-horse", false).await;
    let first = login(&app, "ada@example.com", "correct-horse").await;
    let second = login(&app, "ada@example.com", "correct-horse").await;

    let changed = http_status(
        &app,
        Method::PATCH,
        "/api/v1/users/me/password",
        Some(&first.access_token),
        Some(&serde_json::json!({
            "current_password": "correct-horse",
            "new_password": "a-new-password",
        })),
    )
    .await;

    assert_eq!(changed, StatusCode::OK);

    // Both refresh tokens are gone: whoever prompted the change may already be signed in
    // elsewhere, and those sessions have to go.
    for refresh in [&first.refresh_token, &second.refresh_token] {
        let reused = http_status(
            &app,
            Method::POST,
            "/api/v1/login/refresh",
            None,
            Some(&serde_json::json!({ "refresh_token": refresh })),
        )
        .await;

        assert_eq!(reused, StatusCode::UNAUTHORIZED);
    }

    // And the new password is what signs in afterwards.
    let signed_in = http_status(
        &app,
        Method::POST,
        "/api/v1/login/access-token",
        None,
        Some(&serde_json::json!({
            "email": "ada@example.com",
            "password": "a-new-password",
        })),
    )
    .await;

    assert_eq!(signed_in, StatusCode::OK);
}

#[sqlx::test]
async fn the_wrong_current_password_is_refused(pool: PgPool) {
    let app = app(common::state(&pool).await);
    create_user(&pool, "ada@example.com", "correct-horse", false).await;
    let pair = login(&app, "ada@example.com", "correct-horse").await;

    let code = http_status(
        &app,
        Method::PATCH,
        "/api/v1/users/me/password",
        Some(&pair.access_token),
        Some(&serde_json::json!({
            "current_password": "wrong-password",
            "new_password": "a-new-password",
        })),
    )
    .await;

    assert_eq!(code, StatusCode::UNPROCESSABLE_ENTITY);
}

#[sqlx::test]
async fn an_ordinary_user_cannot_list_everyone(pool: PgPool) {
    let app = app(common::state(&pool).await);
    create_user(&pool, "ada@example.com", "correct-horse", false).await;
    let pair = login(&app, "ada@example.com", "correct-horse").await;

    let code = http_status(
        &app,
        Method::GET,
        "/api/v1/users",
        Some(&pair.access_token),
        None::<&serde_json::Value>,
    )
    .await;

    assert_eq!(code, StatusCode::FORBIDDEN);
}

#[sqlx::test]
async fn a_superuser_can_list_everyone(pool: PgPool) {
    let app = app(common::state(&pool).await);
    create_user(&pool, "admin@example.com", "correct-horse", true).await;
    create_user(&pool, "ada@example.com", "correct-horse", false).await;
    let pair = login(&app, "admin@example.com", "correct-horse").await;

    let (code, body) = json::<serde_json::Value>(
        &app,
        Method::GET,
        "/api/v1/users",
        Some(&pair.access_token),
        None::<&serde_json::Value>,
    )
    .await;

    assert_eq!(code, StatusCode::OK);
    assert!(body["count"].as_i64().unwrap() >= 2);
    assert!(body["data"].as_array().unwrap().len() >= 2);
}

#[sqlx::test]
async fn reading_another_user_is_forbidden_unless_you_are_a_superuser(pool: PgPool) {
    let app = app(common::state(&pool).await);
    let ada = create_user(&pool, "ada@example.com", "correct-horse", false).await;
    create_user(&pool, "grace@example.com", "correct-horse", false).await;
    create_user(&pool, "admin@example.com", "correct-horse", true).await;

    let grace = login(&app, "grace@example.com", "correct-horse").await;
    let admin = login(&app, "admin@example.com", "correct-horse").await;

    // Deliberately a 403 rather than a 404: the id is known to exist, the caller just may not
    // see it. Items mask ownership with 404; users do not.
    let forbidden = http_status(
        &app,
        Method::GET,
        &path("/api/v1/users/{id}", ada.id),
        Some(&grace.access_token),
        None::<&serde_json::Value>,
    )
    .await;

    assert_eq!(forbidden, StatusCode::FORBIDDEN);

    let allowed = http_status(
        &app,
        Method::GET,
        &path("/api/v1/users/{id}", ada.id),
        Some(&admin.access_token),
        None::<&serde_json::Value>,
    )
    .await;

    assert_eq!(allowed, StatusCode::OK);
}

#[sqlx::test]
async fn an_ordinary_user_cannot_create_or_delete_others(pool: PgPool) {
    let app = app(common::state(&pool).await);
    let ada = create_user(&pool, "ada@example.com", "correct-horse", false).await;
    let pair = login(&app, "ada@example.com", "correct-horse").await;

    let create = http_status(
        &app,
        Method::POST,
        "/api/v1/users",
        Some(&pair.access_token),
        Some(&serde_json::json!({
            "email": "new@example.com",
            "password": "correct-horse",
        })),
    )
    .await;

    assert_eq!(create, StatusCode::FORBIDDEN);

    let delete = http_status(
        &app,
        Method::DELETE,
        &path("/api/v1/users/{id}", ada.id),
        Some(&pair.access_token),
        None::<&serde_json::Value>,
    )
    .await;

    // Ordinary users have no DELETE /{id} privilege at all; hitting their own id is still 403.
    assert_eq!(delete, StatusCode::FORBIDDEN);
}

#[sqlx::test]
async fn a_superuser_can_create_and_update_anyone(pool: PgPool) {
    let app = app(common::state(&pool).await);
    create_user(&pool, "admin@example.com", "correct-horse", true).await;
    let pair = login(&app, "admin@example.com", "correct-horse").await;

    let (created_code, created) = json::<serde_json::Value>(
        &app,
        Method::POST,
        "/api/v1/users",
        Some(&pair.access_token),
        Some(&serde_json::json!({
            "email": "grace@example.com",
            "password": "correct-horse",
            "is_superuser": false,
            "is_active": true,
        })),
    )
    .await;

    assert_eq!(created_code, StatusCode::CREATED);
    let id: Uuid = created["id"].as_str().unwrap().parse().unwrap();

    let (updated_code, updated) = json::<serde_json::Value>(
        &app,
        Method::PATCH,
        &path("/api/v1/users/{id}", id),
        Some(&pair.access_token),
        Some(&serde_json::json!({ "full_name": "Grace Hopper", "is_active": false })),
    )
    .await;

    assert_eq!(updated_code, StatusCode::OK);
    assert_eq!(updated["full_name"], "Grace Hopper");
    assert_eq!(updated["is_active"], false);
}

#[sqlx::test]
async fn a_superuser_cannot_delete_themselves(pool: PgPool) {
    let app = app(common::state(&pool).await);
    let admin = create_user(&pool, "admin@example.com", "correct-horse", true).await;
    let pair = login(&app, "admin@example.com", "correct-horse").await;

    let by_id = http_status(
        &app,
        Method::DELETE,
        &path("/api/v1/users/{id}", admin.id),
        Some(&pair.access_token),
        None::<&serde_json::Value>,
    )
    .await;

    assert_eq!(by_id, StatusCode::FORBIDDEN);

    let by_me = http_status(
        &app,
        Method::DELETE,
        "/api/v1/users/me",
        Some(&pair.access_token),
        None::<&serde_json::Value>,
    )
    .await;

    assert_eq!(by_me, StatusCode::FORBIDDEN);
}

#[sqlx::test]
async fn an_ordinary_user_can_delete_themselves(pool: PgPool) {
    let app = app(common::state(&pool).await);
    create_user(&pool, "ada@example.com", "correct-horse", false).await;
    let pair = login(&app, "ada@example.com", "correct-horse").await;

    let code = http_status(
        &app,
        Method::DELETE,
        "/api/v1/users/me",
        Some(&pair.access_token),
        None::<&serde_json::Value>,
    )
    .await;

    assert_eq!(code, StatusCode::OK);

    let signed_in = http_status(
        &app,
        Method::POST,
        "/api/v1/login/access-token",
        None,
        Some(&serde_json::json!({
            "email": "ada@example.com",
            "password": "correct-horse",
        })),
    )
    .await;

    assert_eq!(signed_in, StatusCode::UNAUTHORIZED);
}
