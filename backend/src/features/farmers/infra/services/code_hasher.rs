use sha2::{Digest, Sha256};

use crate::{
    features::farmers::{app::SignInCodeHasher, domain::SignInCode},
    shared::Phone,
};

/// SHA-256 over the server secret, the phone and the code. A six digit code
/// has too few values to hash alone: the secret keeps a leaked table of
/// hashes from being reversed by trying every code.
pub struct Sha256SignInCodeHasher {
    secret: String,
}

impl Sha256SignInCodeHasher {
    pub fn new(secret: String) -> Self {
        Self { secret }
    }
}

impl std::fmt::Debug for Sha256SignInCodeHasher {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("Sha256SignInCodeHasher")
    }
}

impl SignInCodeHasher for Sha256SignInCodeHasher {
    fn hash(&self, phone: &Phone, code: &SignInCode) -> String {
        let mut hasher = Sha256::new();

        hasher.update(self.secret.as_bytes());
        hasher.update(b"|");
        hasher.update(phone.as_str().as_bytes());
        hasher.update(b"|");
        hasher.update(code.as_str().as_bytes());

        hex::encode(hasher.finalize())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn phone(value: &str) -> Phone {
        Phone::new(value.to_string()).expect("phone")
    }

    fn code(value: &str) -> SignInCode {
        SignInCode::new(value.to_string()).expect("code")
    }

    #[test]
    fn the_same_phone_and_code_always_hash_the_same() {
        let hasher = Sha256SignInCodeHasher::new("secret".to_string());

        assert_eq!(
            hasher.hash(&phone("+9647501234567"), &code("123456")),
            hasher.hash(&phone("+9647501234567"), &code("123456"))
        );
    }

    #[test]
    fn another_code_phone_or_secret_gives_another_hash() {
        let hasher = Sha256SignInCodeHasher::new("secret".to_string());
        let hash = hasher.hash(&phone("+9647501234567"), &code("123456"));

        assert_ne!(hash, hasher.hash(&phone("+9647501234567"), &code("123457")));
        assert_ne!(hash, hasher.hash(&phone("+9647501234568"), &code("123456")));
        assert_ne!(
            hash,
            Sha256SignInCodeHasher::new("other".to_string())
                .hash(&phone("+9647501234567"), &code("123456"))
        );
    }

    #[test]
    fn the_hash_does_not_contain_the_code() {
        let hash = Sha256SignInCodeHasher::new("secret".to_string())
            .hash(&phone("+9647501234567"), &code("123456"));

        assert_eq!(hash.len(), 64);
        assert!(!hash.contains("123456"));
    }
}
