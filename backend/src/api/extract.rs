use axum::{
    Json,
    extract::{FromRequest, FromRequestParts, Query, Request, rejection::JsonRejection},
    http::request::Parts,
};
use serde::de::DeserializeOwned;
use validator::{Validate, ValidationErrors, ValidationErrorsKind};

use crate::AppError;

/// A JSON body that has been deserialised *and* validated. Handlers taking this cannot
/// receive a payload that breaks its own rules, so validation is impossible to forget.
#[derive(Debug, Clone, Copy)]
pub struct ValidatedJson<T>(pub T);

impl<T, S> FromRequest<S> for ValidatedJson<T>
where
    T: DeserializeOwned + Validate,
    S: Send + Sync,
{
    type Rejection = AppError;

    async fn from_request(request: Request, state: &S) -> Result<Self, Self::Rejection> {
        let Json(value) = Json::<T>::from_request(request, state)
            .await
            .map_err(malformed_body)?;

        value.validate().map_err(readable_errors)?;
        Ok(Self(value))
    }
}

/// The same for query strings, used by the pagination parameters.
#[derive(Debug, Clone, Copy)]
pub struct ValidatedQuery<T>(pub T);

impl<T, S> FromRequestParts<S> for ValidatedQuery<T>
where
    T: DeserializeOwned + Validate,
    S: Send + Sync,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let Query(value) = Query::<T>::from_request_parts(parts, state)
            .await
            .map_err(|rejection| AppError::validation(rejection.body_text()))?;

        value.validate().map_err(readable_errors)?;
        Ok(Self(value))
    }
}

fn malformed_body(rejection: JsonRejection) -> AppError {
    AppError::validation(rejection.body_text())
}

/// Flattens validator's tree of errors into one line a human can act on, for example
/// `email: must be a valid email address; password: must be 8 to 128 characters`.
fn readable_errors(errors: ValidationErrors) -> AppError {
    let mut lines = Vec::new();
    flatten(&errors, "", &mut lines);

    // The errors are held in a map, so without this the same payload could word its rejection
    // differently between runs.
    lines.sort();

    AppError::validation(lines.join("; "))
}

fn flatten(errors: &ValidationErrors, prefix: &str, lines: &mut Vec<String>) {
    for (field, kind) in errors.errors() {
        // A nested type reports its own failures under an empty key, which belongs to the
        // field holding it rather than to a field of its own.
        let path = match (prefix, field.as_ref()) {
            ("", field) => field.to_owned(),
            (prefix, "") => prefix.to_owned(),
            (prefix, field) => format!("{prefix}.{field}"),
        };

        match kind {
            ValidationErrorsKind::Field(errors) => {
                let reasons: Vec<String> = errors
                    .iter()
                    .map(|error| {
                        error
                            .message
                            .as_ref()
                            .map_or_else(|| error.code.to_string(), ToString::to_string)
                    })
                    .collect();

                lines.push(format!("{path}: {}", reasons.join(", ")));
            }
            ValidationErrorsKind::Struct(nested) => flatten(nested, &path, lines),
            ValidationErrorsKind::List(entries) => {
                for (index, nested) in entries {
                    flatten(nested, &format!("{path}[{index}]"), lines);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::models::{UserRegister, UserUpdate};

    use super::*;

    fn message_for(payload: &str) -> String {
        let parsed: UserRegister = serde_json::from_str(payload).unwrap();
        let errors = parsed.validate().unwrap_err();

        readable_errors(errors).to_string()
    }

    #[test]
    fn a_failure_names_the_field_and_the_reason() {
        let message = message_for(r#"{"email":"nope","password":"password123"}"#);

        assert_eq!(message, "email: must be a valid email address");
    }

    /// The password rule lives on a nested type, so this is the case that would silently
    /// produce an empty message if the flattening only looked one level deep.
    #[test]
    fn a_nested_failure_is_reported_against_the_field_that_holds_it() {
        let message = message_for(r#"{"email":"a@b.com","password":"short"}"#);

        assert_eq!(message, "password: must be 8 to 128 characters");
    }

    #[test]
    fn several_failures_are_listed_in_a_stable_order() {
        let message = message_for(r#"{"email":"nope","password":"short"}"#);

        assert_eq!(
            message,
            "email: must be a valid email address; password: must be 8 to 128 characters"
        );
    }

    #[test]
    fn an_absent_optional_password_raises_nothing() {
        let parsed: UserUpdate = serde_json::from_str(r#"{"full_name":"Ada"}"#).unwrap();

        assert!(parsed.validate().is_ok());
    }
}
