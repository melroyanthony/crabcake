use std::fmt;

use secrecy::{ExposeSecret, SecretString};
use serde::Deserialize;
use validator::{Validate, ValidationError, ValidationErrors};

/// A password on its way in from a request body.
///
/// This exists because `validator`'s derive puts the offending value into the error it
/// produces, which needs `Serialize`, which `SecretString` deliberately does not implement.
/// Rather than unwrap passwords into plain `String`s to satisfy the derive, the rule lives
/// here and the field is validated with `#[validate(nested)]`.
#[derive(Clone, Deserialize)]
#[serde(transparent)]
pub struct Password(SecretString);

/// The nested error is filed under an empty key so that a failure reads `password: ...`
/// rather than `password.value: ...`, which is what a caller expects to see.
const SELF: &str = "";

const MIN_LENGTH: usize = 8;
const MAX_LENGTH: usize = 128;

impl Password {
    pub fn expose(&self) -> &str {
        self.0.expose_secret()
    }
}

impl Validate for Password {
    fn validate(&self) -> Result<(), ValidationErrors> {
        // Counted in characters rather than bytes, so a passphrase of emoji or of any
        // non-Latin script is measured the way its author would measure it.
        let length = self.expose().chars().count();

        if (MIN_LENGTH..=MAX_LENGTH).contains(&length) {
            return Ok(());
        }

        let mut errors = ValidationErrors::new();
        errors.add(
            SELF,
            ValidationError::new("length")
                .with_message(format!("must be {MIN_LENGTH} to {MAX_LENGTH} characters").into()),
        );

        Err(errors)
    }
}

/// Hand-written so that no password reaches a log through a derived `Debug` on some request
/// body that happens to contain one.
impl fmt::Debug for Password {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("Password([redacted])")
    }
}

impl From<&str> for Password {
    fn from(value: &str) -> Self {
        Self(SecretString::from(value))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_short_password_is_rejected() {
        assert!(Password::from("short").validate().is_err());
    }

    #[test]
    fn a_reasonable_password_is_accepted() {
        assert!(Password::from("long enough to count").validate().is_ok());
    }

    #[test]
    fn length_is_counted_in_characters_not_bytes() {
        // Seven characters, twenty-eight bytes: short, however it is stored.
        assert!(Password::from("🦀🦀🦀🦀🦀🦀🦀").validate().is_err());
    }

    #[test]
    fn debug_output_does_not_contain_the_password() {
        let debugged = format!("{:?}", Password::from("hunter2hunter2"));
        assert!(!debugged.contains("hunter2"), "got {debugged}");
    }
}
