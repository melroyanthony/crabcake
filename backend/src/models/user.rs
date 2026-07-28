use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use utoipa::ToSchema;
use uuid::Uuid;
use validator::Validate;

use crate::models::password::Password;

/// A user as stored. Deliberately not `Serialize`: the only way to put a user on the wire is
/// to convert it into [`UserPublic`], which cannot carry the password hash by accident.
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct User {
    pub id: Uuid,
    pub email: String,
    pub hashed_password: String,
    pub full_name: Option<String>,
    pub is_active: bool,
    pub is_superuser: bool,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct UserPublic {
    pub id: Uuid,
    #[schema(example = "ada@example.com")]
    pub email: String,
    #[schema(example = "Ada Lovelace")]
    pub full_name: Option<String>,
    pub is_active: bool,
    pub is_superuser: bool,
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
}

impl From<User> for UserPublic {
    fn from(user: User) -> Self {
        Self {
            id: user.id,
            email: user.email,
            full_name: user.full_name,
            is_active: user.is_active,
            is_superuser: user.is_superuser,
            created_at: user.created_at,
        }
    }
}

/// Open registration. Note the absence of `is_active` and `is_superuser`: a self-registered
/// account cannot promote itself, however the request body is shaped.
#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct UserRegister {
    #[validate(email(message = "must be a valid email address"))]
    #[schema(format = Email, example = "ada@example.com")]
    pub email: String,
    #[validate(nested)]
    pub password: Password,
    #[validate(length(max = 255, message = "must be at most 255 characters"))]
    pub full_name: Option<String>,
}

/// Creation by a superuser, which may set any field.
#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct UserCreate {
    #[validate(email(message = "must be a valid email address"))]
    #[schema(format = Email, example = "ada@example.com")]
    pub email: String,
    #[validate(nested)]
    pub password: Password,
    #[validate(length(max = 255, message = "must be at most 255 characters"))]
    pub full_name: Option<String>,
    #[serde(default = "default_true")]
    pub is_active: bool,
    #[serde(default)]
    pub is_superuser: bool,
}

/// What a user may change about themselves. Notably not their password, which goes through
/// [`PasswordUpdate`] so that the current one has to be proved.
#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct UserUpdateMe {
    #[validate(email(message = "must be a valid email address"))]
    #[schema(format = Email, example = "ada@example.com")]
    pub email: Option<String>,
    #[validate(length(max = 255, message = "must be at most 255 characters"))]
    pub full_name: Option<String>,
}

/// What a superuser may change about anyone, including active status and privileges.
#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct UserUpdate {
    #[validate(email(message = "must be a valid email address"))]
    #[schema(format = Email, example = "ada@example.com")]
    pub email: Option<String>,
    #[validate(nested)]
    pub password: Option<Password>,
    #[validate(length(max = 255, message = "must be at most 255 characters"))]
    pub full_name: Option<String>,
    pub is_active: Option<bool>,
    pub is_superuser: Option<bool>,
}

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct PasswordUpdate {
    pub current_password: Password,
    #[validate(nested)]
    pub new_password: Password,
}

fn default_true() -> bool {
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn registration_cannot_grant_privileges() {
        // A payload that tries to set is_superuser simply loses the field, since
        // UserRegister has nowhere to put it.
        let payload = r#"{"email":"a@b.com","password":"password123","is_superuser":true}"#;
        let parsed: UserRegister = serde_json::from_str(payload).unwrap();

        assert_eq!(parsed.email, "a@b.com");
        assert!(parsed.validate().is_ok());
    }

    #[test]
    fn a_bad_email_and_a_short_password_are_both_reported() {
        let payload = r#"{"email":"not-an-email","password":"short"}"#;
        let parsed: UserRegister = serde_json::from_str(payload).unwrap();
        let errors = parsed.validate().unwrap_err();

        assert_eq!(errors.errors().len(), 2);
    }

    #[test]
    fn a_user_update_without_a_password_is_valid() {
        let parsed: UserUpdate = serde_json::from_str(r#"{"is_active":false}"#).unwrap();

        assert!(parsed.validate().is_ok());
    }
}
