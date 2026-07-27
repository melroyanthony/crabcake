use std::net::SocketAddr;

use figment::{
    Figment,
    providers::{Env, Serialized},
};
use serde::{Deserialize, Serialize};

/// The value the template ships for every secret. Refusing to run with it outside local
/// development is the difference between a template and a security incident.
const PLACEHOLDER_SECRET: &str = "changethis";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Environment {
    Local,
    Staging,
    Production,
}

impl Environment {
    pub fn is_local(self) -> bool {
        self == Self::Local
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    #[serde(default = "defaults::environment")]
    pub environment: Environment,
    #[serde(default = "defaults::project_name")]
    pub project_name: String,
    #[serde(default = "defaults::bind_address")]
    pub bind_address: SocketAddr,

    pub database_url: String,

    #[serde(default = "defaults::frontend_host")]
    pub frontend_host: String,
    /// Comma-separated, because environment variables have no notion of a list.
    #[serde(default)]
    pub cors_origins: String,

    pub secret_key: String,
    #[serde(default = "defaults::access_token_expire_minutes")]
    pub access_token_expire_minutes: i64,
    #[serde(default = "defaults::refresh_token_expire_days")]
    pub refresh_token_expire_days: i64,

    pub first_superuser: String,
    pub first_superuser_password: String,

    #[serde(default = "defaults::request_timeout_seconds")]
    pub request_timeout_seconds: u64,
    #[serde(default = "defaults::body_limit_bytes")]
    pub body_limit_bytes: usize,
}

impl Config {
    /// Reads configuration from the environment, `.env` having already been loaded.
    pub fn from_env() -> Result<Self, ConfigError> {
        let config: Self = Figment::from(Serialized::defaults(figment::value::Dict::new()))
            .merge(Env::raw())
            .extract()
            .map_err(Box::new)?;

        config.guard_against_placeholder_secrets()?;
        Ok(config)
    }

    /// Origins allowed to call the API directly. In the default setup the browser talks to
    /// Next.js rather than here, so this list stays short.
    pub fn cors_origins(&self) -> Vec<&str> {
        self.cors_origins
            .split(',')
            .map(str::trim)
            .filter(|origin| !origin.is_empty())
            .collect()
    }

    fn guard_against_placeholder_secrets(&self) -> Result<(), ConfigError> {
        if self.environment.is_local() {
            return Ok(());
        }

        let unchanged: Vec<&str> = [
            ("SECRET_KEY", &self.secret_key),
            ("FIRST_SUPERUSER_PASSWORD", &self.first_superuser_password),
        ]
        .into_iter()
        .filter(|(_, value)| value.as_str() == PLACEHOLDER_SECRET)
        .map(|(name, _)| name)
        .collect();

        if unchanged.is_empty() {
            Ok(())
        } else {
            Err(ConfigError::PlaceholderSecrets(unchanged.join(", ")))
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    /// Boxed because `figment::Error` is large, and this sits in the `Err` of a `Result`
    /// that callers pass around.
    #[error("invalid configuration: {0}")]
    Invalid(#[from] Box<figment::Error>),
    #[error(
        "{0} still set to the template placeholder. Run `just secrets` and set real values \
         before running outside ENVIRONMENT=local."
    )]
    PlaceholderSecrets(String),
}

mod defaults {
    use std::net::SocketAddr;

    use super::Environment;

    pub fn environment() -> Environment {
        Environment::Local
    }

    pub fn project_name() -> String {
        "App".to_owned()
    }

    pub fn bind_address() -> SocketAddr {
        ([0, 0, 0, 0], 8000).into()
    }

    pub fn frontend_host() -> String {
        "http://localhost:3000".to_owned()
    }

    pub fn access_token_expire_minutes() -> i64 {
        30
    }

    pub fn refresh_token_expire_days() -> i64 {
        30
    }

    pub fn request_timeout_seconds() -> u64 {
        30
    }

    pub fn body_limit_bytes() -> usize {
        2 * 1024 * 1024
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn config(environment: Environment, secret: &str) -> Config {
        Config {
            environment,
            project_name: "Test".to_owned(),
            bind_address: defaults::bind_address(),
            database_url: "postgres://localhost/test".to_owned(),
            frontend_host: defaults::frontend_host(),
            cors_origins: "http://localhost:3000, ,http://localhost:8000".to_owned(),
            secret_key: secret.to_owned(),
            access_token_expire_minutes: 30,
            refresh_token_expire_days: 30,
            first_superuser: "admin@example.com".to_owned(),
            first_superuser_password: "hunter2".to_owned(),
            request_timeout_seconds: 30,
            body_limit_bytes: 1024,
        }
    }

    #[test]
    fn placeholder_secrets_are_allowed_locally() {
        let config = config(Environment::Local, PLACEHOLDER_SECRET);
        assert!(config.guard_against_placeholder_secrets().is_ok());
    }

    #[test]
    fn placeholder_secrets_are_rejected_in_production() {
        let config = config(Environment::Production, PLACEHOLDER_SECRET);
        let error = config.guard_against_placeholder_secrets().unwrap_err();
        assert!(error.to_string().contains("SECRET_KEY"));
    }

    #[test]
    fn cors_origins_ignores_blanks_and_whitespace() {
        let config = config(Environment::Local, "real-secret");
        assert_eq!(
            config.cors_origins(),
            ["http://localhost:3000", "http://localhost:8000"]
        );
    }
}
