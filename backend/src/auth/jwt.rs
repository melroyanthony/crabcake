use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use time::{Duration, OffsetDateTime};
use uuid::Uuid;

use crate::{AppError, AppResult, Config};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    /// Subject: the user this token belongs to.
    pub sub: Uuid,
    /// Expiry, as a Unix timestamp.
    pub exp: i64,
    /// Issued at, as a Unix timestamp.
    pub iat: i64,
}

/// Issues a short-lived access token. Long-lived sessions are the refresh token's job, so
/// that a stolen access token stops working quickly.
pub fn issue_access_token(config: &Config, user_id: Uuid) -> AppResult<String> {
    let now = OffsetDateTime::now_utc();
    let claims = Claims {
        sub: user_id,
        iat: now.unix_timestamp(),
        exp: (now + Duration::minutes(config.access_token_expire_minutes)).unix_timestamp(),
    };

    jsonwebtoken::encode(
        &Header::new(Algorithm::HS256),
        &claims,
        &EncodingKey::from_secret(config.secret_key.as_bytes()),
    )
    .map_err(|error| AppError::Unexpected(anyhow::anyhow!("could not sign token: {error}")))
}

/// Decodes and validates an access token. Every failure, whether a bad signature, a wrong
/// algorithm or an expired token, is reported as `Unauthorized`.
pub fn decode_access_token(config: &Config, token: &str) -> AppResult<Claims> {
    let mut validation = Validation::new(Algorithm::HS256);
    validation.leeway = 0;

    jsonwebtoken::decode::<Claims>(
        token,
        &DecodingKey::from_secret(config.secret_key.as_bytes()),
        &validation,
    )
    .map(|data| data.claims)
    .map_err(|_| AppError::Unauthorized)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Environment;

    fn config(secret: &str, minutes: i64) -> Config {
        Config {
            environment: Environment::Local,
            project_name: "Test".to_owned(),
            bind_address: ([127, 0, 0, 1], 8000).into(),
            database_url: "postgres://localhost/test".to_owned(),
            frontend_host: "http://localhost:3000".to_owned(),
            cors_origins: String::new(),
            secret_key: secret.to_owned(),
            access_token_expire_minutes: minutes,
            refresh_token_expire_days: 30,
            first_superuser: "admin@example.com".to_owned(),
            first_superuser_password: "hunter2".to_owned(),
            request_timeout_seconds: 30,
            body_limit_bytes: 1024,
        }
    }

    #[test]
    fn a_token_round_trips() {
        let config = config("a-secret", 30);
        let user_id = Uuid::new_v4();

        let token = issue_access_token(&config, user_id).unwrap();
        let claims = decode_access_token(&config, &token).unwrap();

        assert_eq!(claims.sub, user_id);
        assert!(claims.exp > claims.iat);
    }

    #[test]
    fn a_token_signed_with_another_secret_is_rejected() {
        let token = issue_access_token(&config("one-secret", 30), Uuid::new_v4()).unwrap();
        let error = decode_access_token(&config("another-secret", 30), &token).unwrap_err();
        assert!(matches!(error, AppError::Unauthorized));
    }

    #[test]
    fn an_expired_token_is_rejected() {
        let config = config("a-secret", -1);
        let token = issue_access_token(&config, Uuid::new_v4()).unwrap();
        let error = decode_access_token(&config, &token).unwrap_err();
        assert!(matches!(error, AppError::Unauthorized));
    }

    #[test]
    fn garbage_is_rejected() {
        let error = decode_access_token(&config("a-secret", 30), "not.a.token").unwrap_err();
        assert!(matches!(error, AppError::Unauthorized));
    }
}
