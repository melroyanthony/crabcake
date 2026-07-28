use std::net::SocketAddr;

use figment::{Figment, providers::Env};
use secrecy::{ExposeSecret, SecretString};
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

/// Every secret is a [`SecretString`], so `Debug` prints `[REDACTED]` and reading the real
/// value takes an explicit `expose_secret()`. That turns "did we ever log the config?" from a
/// question you have to reason about into one you can grep for.
#[derive(Debug, Deserialize)]
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

    pub secret_key: SecretString,
    #[serde(default = "defaults::access_token_expire_minutes")]
    pub access_token_expire_minutes: i64,
    #[serde(default = "defaults::refresh_token_expire_days")]
    pub refresh_token_expire_days: i64,

    pub first_superuser: String,
    pub first_superuser_password: SecretString,

    /// Empty means email is switched off: the worker logs what it would have sent instead of
    /// failing, so a fresh checkout runs without an SMTP server.
    #[serde(default)]
    pub smtp_host: String,
    #[serde(default = "defaults::smtp_port")]
    pub smtp_port: u16,
    #[serde(default)]
    pub smtp_user: String,
    #[serde(default = "defaults::empty_secret")]
    pub smtp_password: SecretString,
    /// Off locally, where Mailcatcher speaks plain SMTP; on everywhere real.
    #[serde(default)]
    pub smtp_tls: bool,
    #[serde(default)]
    pub emails_from_name: String,
    #[serde(default)]
    pub emails_from_email: String,
    #[serde(default = "defaults::password_reset_token_expire_hours")]
    pub password_reset_token_expire_hours: i64,

    /// Empty means uploads are switched off, and the upload endpoints answer 501.
    #[serde(default)]
    pub s3_bucket: String,
    /// Set for anything S3-compatible that is not AWS, such as MinIO locally.
    #[serde(default)]
    pub s3_endpoint: String,
    #[serde(default = "defaults::s3_region")]
    pub s3_region: String,
    #[serde(default)]
    pub s3_access_key_id: String,
    #[serde(default = "defaults::empty_secret")]
    pub s3_secret_access_key: SecretString,
    /// MinIO needs this. Real S3 does not, and rejects it for buckets with dots in the name.
    #[serde(default)]
    pub s3_force_path_style: bool,
    /// How long an upload or download link stays valid.
    #[serde(default = "defaults::upload_url_expire_seconds")]
    pub upload_url_expire_seconds: u64,

    /// Where to send traces, for example `http://otel-collector:4317`. Empty means no export.
    #[serde(default)]
    pub otel_exporter_otlp_endpoint: String,
    /// How this service identifies itself in traces. Falls back to `app`.
    #[serde(default)]
    pub otel_service_name: String,
    #[serde(default)]
    pub metrics_enabled: bool,
    /// Where the metrics endpoint listens. Deliberately not the API's port, and by default not
    /// a public interface either.
    #[serde(default = "defaults::metrics_bind_address")]
    pub metrics_bind_address: SocketAddr,

    /// Requests per second allowed from one caller, or 0 to let everything through.
    #[serde(default = "defaults::rate_limit_per_second")]
    pub rate_limit_per_second: u32,
    /// How far a caller may burst above the steady rate before being turned away.
    #[serde(default = "defaults::rate_limit_burst")]
    pub rate_limit_burst: u32,

    #[serde(default = "defaults::request_timeout_seconds")]
    pub request_timeout_seconds: u64,
    #[serde(default = "defaults::body_limit_bytes")]
    pub body_limit_bytes: usize,
}

impl Config {
    /// Reads configuration from the environment, `.env` having already been loaded.
    pub fn from_env() -> Result<Self, ConfigError> {
        let config: Self = Figment::new()
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

    /// Whether there is anywhere to send mail. Checked rather than assumed, because a template
    /// that panics on startup without an SMTP server would be tiresome to try out.
    pub fn emails_enabled(&self) -> bool {
        !self.smtp_host.is_empty() && !self.emails_from_email.is_empty()
    }

    /// The name mail appears to come from, falling back to the project's own name so that a
    /// half-filled configuration still sends something sensible.
    pub fn emails_from_name(&self) -> &str {
        if self.emails_from_name.is_empty() {
            &self.project_name
        } else {
            &self.emails_from_name
        }
    }

    /// The name traces are attributed to. Without one, every span arrives as
    /// `unknown_service` and a collector cannot tell two services apart.
    /// Whether there is anywhere to put a file.
    pub fn uploads_enabled(&self) -> bool {
        !self.s3_bucket.is_empty()
    }

    pub fn otel_service_name(&self) -> &str {
        if self.otel_service_name.is_empty() {
            "app"
        } else {
            &self.otel_service_name
        }
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
        .filter(|(_, value)| value.expose_secret() == PLACEHOLDER_SECRET)
        .map(|(name, _)| name)
        .collect();

        if unchanged.is_empty() {
            Ok(())
        } else {
            Err(ConfigError::PlaceholderSecrets(unchanged.join(", ")))
        }
    }
}

impl Config {
    /// A valid configuration for tests to start from and then adjust, as
    /// `Config { project_name: "…".to_owned(), ..Config::for_tests() }`.
    ///
    /// Public, and not behind `cfg(test)`, because integration tests compile against this crate
    /// as an ordinary dependency and would otherwise each have to spell out every field.
    pub fn for_tests() -> Self {
        Self {
            environment: Environment::Local,
            project_name: "Test".to_owned(),
            bind_address: defaults::bind_address(),
            database_url: "postgres://localhost/test".to_owned(),
            frontend_host: defaults::frontend_host(),
            cors_origins: String::new(),
            secret_key: SecretString::from("a-test-secret"),
            access_token_expire_minutes: 30,
            refresh_token_expire_days: 30,
            first_superuser: "admin@example.com".to_owned(),
            first_superuser_password: SecretString::from("hunter2"),
            smtp_host: String::new(),
            smtp_port: defaults::smtp_port(),
            smtp_user: String::new(),
            smtp_password: defaults::empty_secret(),
            smtp_tls: false,
            emails_from_name: String::new(),
            emails_from_email: String::new(),
            password_reset_token_expire_hours: 1,
            s3_bucket: String::new(),
            s3_endpoint: String::new(),
            s3_region: defaults::s3_region(),
            s3_access_key_id: String::new(),
            s3_secret_access_key: defaults::empty_secret(),
            s3_force_path_style: false,
            upload_url_expire_seconds: defaults::upload_url_expire_seconds(),
            otel_exporter_otlp_endpoint: String::new(),
            otel_service_name: String::new(),
            metrics_enabled: false,
            metrics_bind_address: defaults::metrics_bind_address(),
            rate_limit_per_second: 0,
            rate_limit_burst: defaults::rate_limit_burst(),
            request_timeout_seconds: 30,
            body_limit_bytes: 1024,
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

    use secrecy::SecretString;

    use super::Environment;

    pub fn empty_secret() -> SecretString {
        SecretString::from("")
    }

    /// Mailcatcher's port, since that is what the local stack runs.
    pub fn smtp_port() -> u16 {
        1025
    }

    pub fn password_reset_token_expire_hours() -> i64 {
        1
    }

    /// Generous enough that ordinary use never notices, low enough to blunt credential
    /// stuffing and scraping.
    pub fn rate_limit_per_second() -> u32 {
        20
    }

    pub fn rate_limit_burst() -> u32 {
        50
    }

    pub fn s3_region() -> String {
        "us-east-1".to_owned()
    }

    /// Long enough for a slow connection to finish a large file, short enough that a link
    /// pasted somewhere it should not be goes stale quickly.
    pub fn upload_url_expire_seconds() -> u64 {
        900
    }

    pub fn environment() -> Environment {
        Environment::Local
    }

    pub fn project_name() -> String {
        "App".to_owned()
    }

    pub fn bind_address() -> SocketAddr {
        ([0, 0, 0, 0], 8000).into()
    }

    /// Loopback, so that metrics are not reachable from another host unless somebody says so.
    /// In Compose this becomes 0.0.0.0 on an unpublished port, which the collector can reach
    /// and the internet cannot.
    pub fn metrics_bind_address() -> SocketAddr {
        ([127, 0, 0, 1], 9100).into()
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

    fn with_placeholder_secret(environment: Environment) -> Config {
        Config {
            environment,
            secret_key: SecretString::from(PLACEHOLDER_SECRET),
            ..Config::for_tests()
        }
    }

    #[test]
    fn placeholder_secrets_are_allowed_locally() {
        let config = with_placeholder_secret(Environment::Local);
        assert!(config.guard_against_placeholder_secrets().is_ok());
    }

    #[test]
    fn placeholder_secrets_are_rejected_in_production() {
        let config = with_placeholder_secret(Environment::Production);
        let error = config.guard_against_placeholder_secrets().unwrap_err();
        assert!(error.to_string().contains("SECRET_KEY"));
    }

    #[test]
    fn cors_origins_ignores_blanks_and_whitespace() {
        let config = Config {
            cors_origins: "http://localhost:3000, ,http://localhost:8000".to_owned(),
            ..Config::for_tests()
        };
        assert_eq!(
            config.cors_origins(),
            ["http://localhost:3000", "http://localhost:8000"]
        );
    }

    #[test]
    fn debug_output_never_contains_a_secret() {
        let config = Config {
            secret_key: SecretString::from("super-secret-signing-key"),
            first_superuser_password: SecretString::from("super-secret-password"),
            ..Config::for_tests()
        };

        let rendered = format!("{config:?}");
        assert!(!rendered.contains("super-secret-signing-key"));
        assert!(!rendered.contains("super-secret-password"));
        assert!(rendered.contains("REDACTED"));
    }
}
