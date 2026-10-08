use crate::{features::fires::domain::FireError, shared::DomainError};

const MAX_LENGTH: usize = 120;

/// The name of the place a fire is at, in one language. It is optional: a
/// detection far from any village has a position and nothing else.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PlaceName(String);

impl PlaceName {
    pub fn new(value: String) -> Result<Self, FireError> {
        let value = value.trim().to_string();

        if value.is_empty() {
            return Err(
                DomainError::InvalidValue("Place name must not be empty".to_string()).into(),
            );
        }

        if value.chars().count() > MAX_LENGTH {
            return Err(DomainError::InvalidValue(format!(
                "Place name must be {MAX_LENGTH} characters max"
            ))
            .into());
        }

        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<&PlaceName> for String {
    fn from(value: &PlaceName) -> Self {
        value.0.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn surrounding_whitespace_is_stripped_before_storing() {
        assert_eq!(
            PlaceName::new("  Near Chamchamal  ".to_string())
                .expect("place")
                .as_str(),
            "Near Chamchamal"
        );
    }

    #[test]
    fn a_name_of_only_whitespace_is_empty_not_valid() {
        assert!(PlaceName::new("   ".to_string()).is_err());
    }

    #[test]
    fn the_limit_counts_characters_not_bytes() {
        assert!(
            PlaceName::new("ک".repeat(MAX_LENGTH)).is_ok(),
            "a Sorani name must not be rejected for its byte length"
        );
        assert!(PlaceName::new("ک".repeat(MAX_LENGTH + 1)).is_err());
    }
}
