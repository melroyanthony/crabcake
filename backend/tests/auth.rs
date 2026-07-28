//! Auth over HTTP: signing in, refreshing, signing out, and the extractors that everything
//! else hangs off.

mod common;

use app::services::auth::TokenPair;
use axum::http::{Method, StatusCode};
use common::{app, create_inactive, create_user, http_status, json, login, problem};
use sqlx::PgPool;

#[sqlx::test]
async fn a_known_password_yields_a_token_pair(pool: PgPool) {
    let app = app(common::state(&pool).await);
    create_user(&pool, "ada@example.com", "correct-horse", false).await;

    let (code, pair) = json::<TokenPair>(
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

    assert_eq!(code, StatusCode::OK);
    assert_eq!(pair.token_type, "bearer");
    assert!(!pair.access_token.is_empty());
    assert!(!pair.refresh_token.is_empty());
}

#[sqlx::test]
async fn a_wrong_password_is_a_401(pool: PgPool) {
    let app = app(common::state(&pool).await);
    create_user(&pool, "ada@example.com", "correct-horse", false).await;

    let (code, body) = problem(
        &app,
        Method::POST,
        "/api/v1/login/access-token",
        None,
        Some(&serde_json::json!({
            "email": "ada@example.com",
            "password": "wrong-password",
        })),
    )
    .await;

    assert_eq!(code, StatusCode::UNAUTHORIZED);
    assert_eq!(body["title"], "Unauthorized");
}

#[sqlx::test]
async fn an_unknown_address_is_also_a_401(pool: PgPool) {
    let app = app(common::state(&pool).await);

    let code = http_status(
        &app,
        Method::POST,
        "/api/v1/login/access-token",
        None,
        Some(&serde_json::json!({
            "email": "nobody@example.com",
            "password": "anything-at-all",
        })),
    )
    .await;

    // Same status as a wrong password, so the caller cannot tell the two apart.
    assert_eq!(code, StatusCode::UNAUTHORIZED);
}

#[sqlx::test]
async fn a_deactivated_account_cannot_sign_in(pool: PgPool) {
    let app = app(common::state(&pool).await);
    create_inactive(&pool, "ada@example.com", "correct-horse").await;

    let (code, body) = problem(
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

    assert_eq!(code, StatusCode::FORBIDDEN);
    assert_eq!(body["title"], "Forbidden");
}

#[sqlx::test]
async fn refreshing_rotates_the_refresh_token(pool: PgPool) {
    let app = app(common::state(&pool).await);
    create_user(&pool, "ada@example.com", "correct-horse", false).await;
    let first = login(&app, "ada@example.com", "correct-horse").await;

    let (code, second) = json::<TokenPair>(
        &app,
        Method::POST,
        "/api/v1/login/refresh",
        None,
        Some(&serde_json::json!({ "refresh_token": first.refresh_token })),
    )
    .await;

    assert_eq!(code, StatusCode::OK);
    // Access tokens can collide when issued in the same second; the refresh token is what
    // must rotate, because that is what makes a stolen one good for at most one exchange.
    assert_ne!(second.refresh_token, first.refresh_token);
    assert!(!second.access_token.is_empty());

    // The old refresh token is spent: presenting it again is a 401, not a second pair.
    let reused = http_status(
        &app,
        Method::POST,
        "/api/v1/login/refresh",
        None,
        Some(&serde_json::json!({ "refresh_token": first.refresh_token })),
    )
    .await;

    assert_eq!(reused, StatusCode::UNAUTHORIZED);
}

#[sqlx::test]
async fn logout_makes_the_refresh_token_useless(pool: PgPool) {
    let app = app(common::state(&pool).await);
    create_user(&pool, "ada@example.com", "correct-horse", false).await;
    let pair = login(&app, "ada@example.com", "correct-horse").await;

    let logged_out = http_status(
        &app,
        Method::POST,
        "/api/v1/login/logout",
        None,
        Some(&serde_json::json!({ "refresh_token": pair.refresh_token })),
    )
    .await;

    assert_eq!(logged_out, StatusCode::OK);

    let reused = http_status(
        &app,
        Method::POST,
        "/api/v1/login/refresh",
        None,
        Some(&serde_json::json!({ "refresh_token": pair.refresh_token })),
    )
    .await;

    assert_eq!(reused, StatusCode::UNAUTHORIZED);
}

#[sqlx::test]
async fn logging_out_an_unknown_token_is_still_a_success(pool: PgPool) {
    let app = app(common::state(&pool).await);

    // Quiet success: a stolen token that has already been revoked, or a typo, must not tell
    // the caller anything useful about what the server knows.
    let code = http_status(
        &app,
        Method::POST,
        "/api/v1/login/logout",
        None,
        Some(&serde_json::json!({ "refresh_token": "not-a-real-token" })),
    )
    .await;

    assert_eq!(code, StatusCode::OK);
}

#[sqlx::test]
async fn logout_everywhere_ends_every_session(pool: PgPool) {
    let app = app(common::state(&pool).await);
    create_user(&pool, "ada@example.com", "correct-horse", false).await;
    let first = login(&app, "ada@example.com", "correct-horse").await;
    let second = login(&app, "ada@example.com", "correct-horse").await;

    let code = http_status(
        &app,
        Method::POST,
        "/api/v1/login/logout-everywhere",
        Some(&first.access_token),
        None::<&serde_json::Value>,
    )
    .await;

    assert_eq!(code, StatusCode::OK);

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
}

#[sqlx::test]
async fn test_token_returns_the_caller(pool: PgPool) {
    let app = app(common::state(&pool).await);
    let user = create_user(&pool, "ada@example.com", "correct-horse", false).await;
    let pair = login(&app, "ada@example.com", "correct-horse").await;

    let (code, body) = json::<serde_json::Value>(
        &app,
        Method::POST,
        "/api/v1/login/test-token",
        Some(&pair.access_token),
        None::<&serde_json::Value>,
    )
    .await;

    assert_eq!(code, StatusCode::OK);
    assert_eq!(body["id"], user.id.to_string());
    assert_eq!(body["email"], "ada@example.com");
}

#[sqlx::test]
async fn a_missing_token_is_unauthorized(pool: PgPool) {
    let app = app(common::state(&pool).await);

    let code = http_status(
        &app,
        Method::POST,
        "/api/v1/login/test-token",
        None,
        None::<&serde_json::Value>,
    )
    .await;

    assert_eq!(code, StatusCode::UNAUTHORIZED);
}

#[sqlx::test]
async fn a_forged_token_is_unauthorized(pool: PgPool) {
    let app = app(common::state(&pool).await);

    let code = http_status(
        &app,
        Method::POST,
        "/api/v1/login/test-token",
        Some("this.is.not.a.jwt"),
        None::<&serde_json::Value>,
    )
    .await;

    assert_eq!(code, StatusCode::UNAUTHORIZED);
}

#[sqlx::test]
async fn a_deactivated_user_cannot_use_an_old_access_token(pool: PgPool) {
    let app = app(common::state(&pool).await);
    let user = create_user(&pool, "ada@example.com", "correct-horse", false).await;
    let pair = login(&app, "ada@example.com", "correct-horse").await;

    sqlx::query("update users set is_active = false where id = $1")
        .bind(user.id)
        .execute(&pool)
        .await
        .expect("the account is deactivated");

    let (code, body) = problem(
        &app,
        Method::POST,
        "/api/v1/login/test-token",
        Some(&pair.access_token),
        None::<&serde_json::Value>,
    )
    .await;

    assert_eq!(code, StatusCode::FORBIDDEN);
    assert_eq!(body["title"], "Forbidden");
}
