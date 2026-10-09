use argon2::{
    Argon2, PasswordHasher as _, PasswordVerifier as _,
    password_hash::{PasswordHash as EncodedHash, SaltString},
};
use async_trait::async_trait;

use crate::{
    app::AppError as GlobalAppError,
    features::staff::{
        app::{AppError, PasswordHasher},
        domain::{Password, PasswordHash},
    },
};

const SALT_BYTES: usize = 16;

/// Argon2id with the crate's default cost (19 MiB of memory, 2 passes),
/// stored in the PHC text form, which carries the parameters and the salt.
/// Hashing is slow on purpose, so it runs off the async threads.
pub struct Argon2idPasswordHasher {
    /// A hash of a password nobody knows. A sign-in for an email without an
    /// account is checked against it, so it costs what a real check costs.
    decoy: String,
}

impl Argon2idPasswordHasher {
    pub fn new() -> Result<Self, AppError> {
        let mut unknown = [0u8; 32];
        rand::fill(&mut unknown);

        Ok(Self {
            decoy: encode(&hex::encode(unknown))?,
        })
    }
}

impl std::fmt::Debug for Argon2idPasswordHasher {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("Argon2idPasswordHasher")
    }
}

fn hashing_failed(error: impl std::fmt::Display) -> AppError {
    tracing::error!(%error, "password hashing failed");

    GlobalAppError::InternalServerError.into()
}

fn encode(password: &str) -> Result<String, AppError> {
    let mut salt = [0u8; SALT_BYTES];
    rand::fill(&mut salt);

    let salt = SaltString::encode_b64(&salt).map_err(hashing_failed)?;

    Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        .map(|hash| hash.to_string())
        .map_err(hashing_failed)
}

fn matches(password: &str, encoded: &str) -> bool {
    match EncodedHash::new(encoded) {
        Ok(parsed) => Argon2::default()
            .verify_password(password.as_bytes(), &parsed)
            .is_ok(),
        Err(error) => {
            tracing::error!(%error, "a stored password hash cannot be read");

            false
        }
    }
}

#[async_trait]
impl PasswordHasher for Argon2idPasswordHasher {
    async fn hash(&self, password: &Password) -> Result<PasswordHash, AppError> {
        let password = password.as_str().to_string();

        let encoded = tokio::task::spawn_blocking(move || encode(&password))
            .await
            .map_err(hashing_failed)??;

        Ok(PasswordHash::new(encoded)?)
    }

    async fn verify(
        &self,
        password: &Password,
        stored: Option<&PasswordHash>,
    ) -> Result<bool, AppError> {
        let password = password.as_str().to_string();
        let known = stored.is_some();
        let encoded = stored
            .map(|stored| stored.as_str().to_string())
            .unwrap_or_else(|| self.decoy.clone());

        let matched = tokio::task::spawn_blocking(move || matches(&password, &encoded))
            .await
            .map_err(hashing_failed)?;

        Ok(known && matched)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn password(value: &str) -> Password {
        Password::presented(value.to_string()).expect("password")
    }

    #[tokio::test]
    async fn a_hash_is_argon2id_and_does_not_contain_the_password() {
        let hasher = Argon2idPasswordHasher::new().expect("hasher");

        let hash = hasher
            .hash(&password("correct horse battery"))
            .await
            .expect("hash");

        assert!(hash.as_str().starts_with("$argon2id$"));
        assert!(!hash.as_str().contains("correct horse battery"));
    }

    #[tokio::test]
    async fn the_right_password_verifies_and_a_wrong_one_does_not() {
        let hasher = Argon2idPasswordHasher::new().expect("hasher");
        let hash = hasher
            .hash(&password("correct horse battery"))
            .await
            .expect("hash");

        assert!(
            hasher
                .verify(&password("correct horse battery"), Some(&hash))
                .await
                .expect("verified")
        );
        assert!(
            !hasher
                .verify(&password("correct horse batterz"), Some(&hash))
                .await
                .expect("verified")
        );
    }

    #[tokio::test]
    async fn the_same_password_hashes_differently_each_time() {
        let hasher = Argon2idPasswordHasher::new().expect("hasher");

        let first = hasher.hash(&password("correct horse battery")).await;
        let second = hasher.hash(&password("correct horse battery")).await;

        assert_ne!(
            first.expect("hash").as_str(),
            second.expect("hash").as_str(),
            "without a fresh salt equal passwords would show as equal hashes"
        );
    }

    #[tokio::test]
    async fn with_no_stored_hash_nothing_verifies() {
        let hasher = Argon2idPasswordHasher::new().expect("hasher");

        assert!(
            !hasher
                .verify(&password("anything at all"), None)
                .await
                .expect("verified")
        );
    }

    #[tokio::test]
    async fn a_stored_hash_that_cannot_be_read_verifies_nothing() {
        let hasher = Argon2idPasswordHasher::new().expect("hasher");
        let broken = PasswordHash::new("not-a-hash".to_string()).expect("hash");

        assert!(
            !hasher
                .verify(&password("anything at all"), Some(&broken))
                .await
                .expect("verified")
        );
    }
}
