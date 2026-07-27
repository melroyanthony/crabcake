use argon2::{
    Argon2,
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString, rand_core::OsRng},
};

use crate::{AppError, AppResult};

/// Hashes a password with Argon2id and a fresh random salt. Returns a PHC string, which
/// carries the algorithm and parameters, so the cost can be raised later without invalidating
/// existing hashes.
pub fn hash(password: &str) -> AppResult<String> {
    let salt = SaltString::generate(&mut OsRng);

    Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        .map(|hash| hash.to_string())
        .map_err(|error| AppError::Unexpected(anyhow::anyhow!("could not hash password: {error}")))
}

/// Verifies a password against a stored hash. A malformed hash is a failed verification, not
/// an error, so a corrupt row cannot be told apart from a wrong password by an attacker.
pub fn verify(password: &str, hashed: &str) -> bool {
    let Ok(parsed) = PasswordHash::new(hashed) else {
        tracing::error!("stored password hash is malformed");
        return false;
    };

    Argon2::default()
        .verify_password(password.as_bytes(), &parsed)
        .is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_password_verifies_against_its_own_hash() {
        let hashed = hash("correct horse battery staple").unwrap();
        assert!(verify("correct horse battery staple", &hashed));
    }

    #[test]
    fn a_wrong_password_does_not_verify() {
        let hashed = hash("correct horse battery staple").unwrap();
        assert!(!verify("Tr0ub4dor&3", &hashed));
    }

    #[test]
    fn the_same_password_hashes_differently_each_time() {
        let first = hash("same input").unwrap();
        let second = hash("same input").unwrap();
        assert_ne!(first, second, "salts should make hashes unique");
    }

    #[test]
    fn a_malformed_hash_fails_closed() {
        assert!(!verify("anything", "not-a-phc-string"));
    }
}
