use crate::{features::alwa::domain::AlwaError, shared::DomainError};

const MAX_LENGTH: usize = 128;

/// The key the app sends with a new listing so that a retry after a lost
/// answer returns the listing already posted instead of posting a second one.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IdempotencyKey(String);

impl IdempotencyKey {
    pub fn new(value: String) -> Result<Self, AlwaError> {
        let value = value.trim().to_string();

        if value.is_empty() || value.chars().count() > MAX_LENGTH {
            return Err(DomainError::InvalidValue(format!(
                "Idempotency key must be 1 to {MAX_LENGTH} characters"
            ))
            .into());
        }

        if !value.chars().all(|character| character.is_ascii_graphic()) {
            return Err(DomainError::InvalidValue(
                "Idempotency key must be printable ASCII without spaces".to_string(),
            )
            .into());
        }

        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<&IdempotencyKey> for String {
    fn from(value: &IdempotencyKey) -> Self {
        value.0.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_a_uuid() {
        assert!(IdempotencyKey::new("3f0e8c1a-9b7d-4c55-a1f2-0d9e6b7c8a90".to_string()).is_ok());
    }

    #[test]
    fn rejects_an_empty_or_over_long_key() {
        assert!(IdempotencyKey::new("  ".to_string()).is_err());
        assert!(IdempotencyKey::new("k".repeat(MAX_LENGTH)).is_ok());
        assert!(IdempotencyKey::new("k".repeat(MAX_LENGTH + 1)).is_err());
    }

    #[test]
    fn rejects_spaces_inside_the_key() {
        assert!(IdempotencyKey::new("two words".to_string()).is_err());
    }
}
