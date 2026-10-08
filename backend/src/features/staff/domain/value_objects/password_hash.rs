use crate::{features::staff::domain::StaffError, shared::DomainError};

/// The stored form of a password: the hasher's own encoded text, which
/// carries the algorithm, its parameters and the salt.
#[derive(Clone, PartialEq, Eq)]
pub struct PasswordHash(String);

impl PasswordHash {
    pub fn new(value: String) -> Result<Self, StaffError> {
        if value.is_empty() {
            return Err(
                DomainError::InvalidValue("Password hash must not be empty".to_string()).into(),
            );
        }

        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// A hash can be attacked offline, so it stays out of debug output too.
impl std::fmt::Debug for PasswordHash {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("PasswordHash(******)")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_empty_hash_is_rejected() {
        assert!(PasswordHash::new(String::new()).is_err());
    }

    #[test]
    fn debug_output_never_shows_the_hash() {
        let shown = format!(
            "{:?}",
            PasswordHash::new("$argon2id$v=19$abcdef".to_string()).expect("hash")
        );

        assert!(!shown.contains("argon2id"));
        assert!(!shown.contains("abcdef"));
    }
}
