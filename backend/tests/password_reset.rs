//! The password reset flow, end to end through the database and the job queue.
//!
//! Every test gets its own database, created and dropped by `#[sqlx::test]`, so these run in
//! parallel and none of them can see another's rows.

use app::{
    AppState, Config,
    auth::password,
    jobs,
    models::Password,
    repo::{self, users::NewUser},
    services::password_reset,
};
use sqlx::PgPool;
use uuid::Uuid;

/// Builds state around the test's own database. The queue's tables are not part of the app's
/// migrations, so they are created here the same way the binaries create them on startup.
async fn state(pool: &PgPool) -> AppState {
    jobs::setup(pool).await.expect("the queue tables");

    let config = Config {
        frontend_host: "https://app.test".to_owned(),
        password_reset_token_expire_hours: 1,
        ..Config::for_tests()
    };

    AppState::new(config, pool.clone(), jobs::queue(pool.clone()))
}

async fn user(pool: &PgPool, email: &str, plaintext: &str) -> Uuid {
    let hashed = password::hash(plaintext).expect("hashing works");

    repo::users::create(
        pool,
        NewUser {
            email,
            hashed_password: &hashed,
            full_name: None,
            is_active: true,
            is_superuser: false,
        },
    )
    .await
    .expect("the user is created")
    .id
}

/// Reads the reset link out of the queued email, which is the only place the token exists in
/// the clear: the database holds a digest, exactly as an attacker with a dump would find it.
async fn queued_reset_token(pool: &PgPool) -> String {
    let payload: serde_json::Value =
        sqlx::query_scalar("select job from apalis.jobs order by run_at desc limit 1")
            .fetch_one(pool)
            .await
            .expect("an email was queued");

    let text = payload["text"].as_str().expect("the email has a text part");

    text.split("token=")
        .nth(1)
        .expect("the email carries a reset link")
        .split_whitespace()
        .next()
        .expect("the link ends somewhere")
        .to_owned()
}

#[sqlx::test]
async fn a_reset_replaces_the_password(pool: PgPool) {
    let state = state(&pool).await;
    let id = user(&pool, "ada@example.com", "the-old-password").await;

    password_reset::request(&state, "ada@example.com")
        .await
        .expect("the request is accepted");

    let token = queued_reset_token(&pool).await;

    password_reset::confirm(&state, &token, &Password::from("the-new-password"))
        .await
        .expect("the reset succeeds");

    let stored = repo::users::find_by_id(&pool, id)
        .await
        .expect("the query runs")
        .expect("the user is still there");

    assert!(password::verify(
        "the-new-password",
        &stored.hashed_password
    ));
    assert!(!password::verify(
        "the-old-password",
        &stored.hashed_password
    ));
}

/// A reset link in an inbox should not stay a spare key to the account.
#[sqlx::test]
async fn a_link_works_only_once(pool: PgPool) {
    let state = state(&pool).await;
    user(&pool, "ada@example.com", "the-old-password").await;

    password_reset::request(&state, "ada@example.com")
        .await
        .unwrap();

    let token = queued_reset_token(&pool).await;

    password_reset::confirm(&state, &token, &Password::from("the-new-password"))
        .await
        .expect("the first use succeeds");

    let second = password_reset::confirm(&state, &token, &Password::from("a-third-password")).await;

    assert!(second.is_err(), "the same link was accepted twice");
}

/// Asking again should invalidate the first email, so that a leaked or forwarded one stops
/// working as soon as the real owner asks for another.
#[sqlx::test]
async fn asking_again_invalidates_the_earlier_link(pool: PgPool) {
    let state = state(&pool).await;
    user(&pool, "ada@example.com", "the-old-password").await;

    password_reset::request(&state, "ada@example.com")
        .await
        .unwrap();
    let first = queued_reset_token(&pool).await;

    password_reset::request(&state, "ada@example.com")
        .await
        .unwrap();
    let second = queued_reset_token(&pool).await;

    assert_ne!(first, second, "the same token was emailed twice");

    assert!(
        password_reset::confirm(&state, &first, &Password::from("the-new-password"))
            .await
            .is_err(),
        "the superseded link still worked"
    );

    assert!(
        password_reset::confirm(&state, &second, &Password::from("the-new-password"))
            .await
            .is_ok()
    );
}

/// Whoever asked for the reset may well be signed in already.
#[sqlx::test]
async fn a_reset_ends_every_session(pool: PgPool) {
    let state = state(&pool).await;
    let id = user(&pool, "ada@example.com", "the-old-password").await;

    let expires_at = time::OffsetDateTime::now_utc() + time::Duration::days(30);
    repo::refresh_tokens::insert(&pool, id, "a-session-digest", expires_at)
        .await
        .unwrap();

    assert!(
        repo::refresh_tokens::find_active(&pool, "a-session-digest")
            .await
            .unwrap()
            .is_some()
    );

    password_reset::request(&state, "ada@example.com")
        .await
        .unwrap();
    let token = queued_reset_token(&pool).await;

    password_reset::confirm(&state, &token, &Password::from("the-new-password"))
        .await
        .unwrap();

    assert!(
        repo::refresh_tokens::find_active(&pool, "a-session-digest")
            .await
            .unwrap()
            .is_none(),
        "a session survived the reset"
    );
}

/// The endpoint answers the same way whatever it finds, and it must also do the same amount of
/// visible work: queueing nothing for an unknown address is what keeps it from being a way to
/// ask whether somebody has an account.
#[sqlx::test]
async fn an_unknown_address_is_accepted_and_queues_nothing(pool: PgPool) {
    let state = state(&pool).await;

    password_reset::request(&state, "nobody@example.com")
        .await
        .expect("the request is accepted");

    let queued: i64 = sqlx::query_scalar("select count(*) from apalis.jobs")
        .fetch_one(&pool)
        .await
        .unwrap();

    assert_eq!(queued, 0);
}

#[sqlx::test]
async fn a_deactivated_account_gets_no_link(pool: PgPool) {
    let state = state(&pool).await;
    let id = user(&pool, "ada@example.com", "the-old-password").await;

    sqlx::query!("update users set is_active = false where id = $1", id)
        .execute(&pool)
        .await
        .unwrap();

    password_reset::request(&state, "ada@example.com")
        .await
        .expect("the request is accepted");

    let queued: i64 = sqlx::query_scalar("select count(*) from apalis.jobs")
        .fetch_one(&pool)
        .await
        .unwrap();

    assert_eq!(queued, 0);
}

#[sqlx::test]
async fn an_invented_token_is_refused(pool: PgPool) {
    let state = state(&pool).await;
    user(&pool, "ada@example.com", "the-old-password").await;

    let result = password_reset::confirm(
        &state,
        "not-a-real-token",
        &Password::from("a-new-password"),
    )
    .await;

    assert!(result.is_err());
}

/// Expiry is enforced by the database rather than by comparing timestamps in Rust, so a link is
/// dead once its hour is up even if the application's clock has drifted.
#[sqlx::test]
async fn an_expired_link_is_refused(pool: PgPool) {
    let state = state(&pool).await;
    user(&pool, "ada@example.com", "the-old-password").await;

    password_reset::request(&state, "ada@example.com")
        .await
        .unwrap();
    let token = queued_reset_token(&pool).await;

    sqlx::query!("update password_reset_tokens set expires_at = now() - interval '1 minute'")
        .execute(&pool)
        .await
        .unwrap();

    let result = password_reset::confirm(&state, &token, &Password::from("a-new-password")).await;

    assert!(result.is_err(), "an expired link was accepted");
}
