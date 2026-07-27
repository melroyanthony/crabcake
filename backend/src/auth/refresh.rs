use argon2::password_hash::rand_core::{OsRng, RngCore};
use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
use sha2::{Digest, Sha256};

/// A freshly minted refresh token: the value handed to the client, and the digest that is all
/// the database ever sees.
pub struct NewRefreshToken {
    pub plaintext: String,
    pub digest: String,
}

/// Refresh tokens are opaque random bytes rather than signed JWTs, because they need to be
/// revocable, and you cannot revoke something you never wrote down.
pub fn generate() -> NewRefreshToken {
    let mut bytes = [0u8; 32];
    OsRng.fill_bytes(&mut bytes);

    let plaintext = URL_SAFE_NO_PAD.encode(bytes);
    let digest = digest(&plaintext);

    NewRefreshToken { plaintext, digest }
}

/// A plain SHA-256 rather than a password hash: the input already has 256 bits of entropy, so
/// there is nothing for a slow hash to protect against, and lookups happen on every refresh.
pub fn digest(token: &str) -> String {
    URL_SAFE_NO_PAD.encode(Sha256::digest(token.as_bytes()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tokens_are_unique() {
        assert_ne!(generate().plaintext, generate().plaintext);
    }

    #[test]
    fn the_digest_matches_the_plaintext() {
        let token = generate();
        assert_eq!(digest(&token.plaintext), token.digest);
    }

    #[test]
    fn the_digest_is_not_the_token() {
        let token = generate();
        assert_ne!(token.digest, token.plaintext);
    }
}
