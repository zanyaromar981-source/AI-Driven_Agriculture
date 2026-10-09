use crate::{features::alerts::domain::AlertError, shared::DomainError};

const MAX_LENGTH: usize = 4096;

/// The address the push service gave one phone. It is a secret: whoever
/// holds it can send that phone a notification, so it never appears in logs
/// and `Debug` hides it.
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct PushToken(String);

impl PushToken {
    pub fn new(value: String) -> Result<Self, AlertError> {
        let length = value.chars().count();

        if length == 0 || length > MAX_LENGTH {
            return Err(DomainError::InvalidValue(format!(
                "Push token must be 1 to {MAX_LENGTH} characters"
            ))
            .into());
        }

        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Debug for PushToken {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("PushToken(hidden)")
    }
}

impl From<&PushToken> for String {
    fn from(value: &PushToken) -> Self {
        value.0.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_limit_is_inclusive_and_counts_characters() {
        assert!(PushToken::new("a".repeat(MAX_LENGTH)).is_ok());
        assert!(PushToken::new("a".repeat(MAX_LENGTH + 1)).is_err());
        assert!(PushToken::new(String::new()).is_err());
    }

    #[test]
    fn debug_output_never_shows_the_token() {
        let token = PushToken::new("fcm-secret-token".to_string()).expect("token");

        assert!(!format!("{token:?}").contains("secret"));
    }
}
