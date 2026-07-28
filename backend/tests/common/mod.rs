//! Shared helpers for HTTP integration tests.
//!
//! Every test gets its own database from `#[sqlx::test]`. The app is exercised through
//! `api::build` and `oneshot`, rather than a bound listener: that keeps ConnectInfo and a
//! free port out of the picture, and `Config::for_tests` already turns rate limiting and
//! metrics off so neither gets in the way.
//!
//! Each suite only uses some of these, so the unused ones would otherwise warn in every
//! binary that does not happen to need them.
#![allow(dead_code)]

use app::{
    AppState, Config, api,
    auth::password,
    jobs,
    models::User,
    repo::{self, users::NewUser},
    services::auth::TokenPair,
    storage::Storage,
};
use axum::{
    Router,
    body::{Body, to_bytes},
    http::{Method, Request, StatusCode, header},
    response::Response,
};
use serde::Serialize;
use serde::de::DeserializeOwned;
use sqlx::PgPool;
use tower::ServiceExt as _;
use uuid::Uuid;

/// Builds state around the test's own database. The queue's tables are not part of the app's
/// migrations, so they are created here the same way the binaries create them on startup.
pub async fn state(pool: &PgPool) -> AppState {
    jobs::setup(pool).await.expect("the queue tables");

    let config = Config::for_tests();
    let storage = Storage::from_config(&config).await;

    AppState::new(config, pool.clone(), jobs::queue(pool.clone()), storage)
}

pub fn app(state: AppState) -> Router {
    api::build(state)
}

/// Creates a user directly in the database. Prefer this over signup when the test needs a
/// known password hash, an inactive account, or a superuser.
pub async fn create_user(pool: &PgPool, email: &str, plaintext: &str, is_superuser: bool) -> User {
    let hashed = password::hash(plaintext).expect("hashing works");

    repo::users::create(
        pool,
        NewUser {
            email,
            hashed_password: &hashed,
            full_name: None,
            is_active: true,
            is_superuser,
        },
    )
    .await
    .expect("the user is created")
}

pub async fn create_inactive(pool: &PgPool, email: &str, plaintext: &str) -> User {
    let hashed = password::hash(plaintext).expect("hashing works");

    repo::users::create(
        pool,
        NewUser {
            email,
            hashed_password: &hashed,
            full_name: None,
            is_active: false,
            is_superuser: false,
        },
    )
    .await
    .expect("the user is created")
}

/// Signs in through the API and returns the token pair the rest of the suite will use.
pub async fn login(app: &Router, email: &str, password: &str) -> TokenPair {
    let (status, pair) = json::<TokenPair>(
        app,
        Method::POST,
        "/api/v1/login/access-token",
        None,
        Some(&serde_json::json!({ "email": email, "password": password })),
    )
    .await;

    assert_eq!(status, StatusCode::OK, "login should succeed for {email}");
    pair
}

pub async fn call(
    app: &Router,
    method: Method,
    path: &str,
    token: Option<&str>,
    body: Option<Body>,
) -> Response {
    let mut builder = Request::builder().method(method).uri(path);

    if let Some(token) = token {
        builder = builder.header(header::AUTHORIZATION, format!("Bearer {token}"));
    }

    if body.is_some() {
        builder = builder.header(header::CONTENT_TYPE, "application/json");
    }

    let request = builder
        .body(body.unwrap_or_else(Body::empty))
        .expect("the request is valid");

    app.clone()
        .oneshot(request)
        .await
        .expect("the router answers")
}

/// Sends a JSON body and deserialises the response. Panics if the body is not the type asked
/// for, so a 401 that was supposed to be a `TokenPair` fails the test rather than later.
pub async fn json<T: DeserializeOwned>(
    app: &Router,
    method: Method,
    path: &str,
    token: Option<&str>,
    body: Option<&impl Serialize>,
) -> (StatusCode, T) {
    let body =
        body.map(|value| Body::from(serde_json::to_vec(value).expect("the body serialises")));

    let response = call(app, method, path, token, body).await;
    let status = response.status();
    let bytes = to_bytes(response.into_body(), 1024 * 1024)
        .await
        .expect("the body is readable");

    let value = serde_json::from_slice(&bytes).unwrap_or_else(|error| {
        panic!(
            "could not decode {status} body as {}: {error}; body was {}",
            std::any::type_name::<T>(),
            String::from_utf8_lossy(&bytes)
        )
    });

    (status, value)
}

/// The response status alone. Named so it does not collide with a local `status` binding at
/// the call site, which is easy to write when unpacking `(status, body)`.
pub async fn http_status(
    app: &Router,
    method: Method,
    path: &str,
    token: Option<&str>,
    body: Option<&impl Serialize>,
) -> StatusCode {
    let body =
        body.map(|value| Body::from(serde_json::to_vec(value).expect("the body serialises")));

    call(app, method, path, token, body).await.status()
}

pub async fn problem(
    app: &Router,
    method: Method,
    path: &str,
    token: Option<&str>,
    body: Option<&impl Serialize>,
) -> (StatusCode, serde_json::Value) {
    json(app, method, path, token, body).await
}

/// A path that embeds a UUID without forcing every call site to format it.
pub fn path(template: &str, id: Uuid) -> String {
    template.replace("{id}", &id.to_string())
}
