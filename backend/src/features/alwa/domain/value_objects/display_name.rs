use crate::{features::alwa::domain::AlwaError, shared::DomainError};

const MAX_LENGTH: usize = 60;

/// The name a seller or a buyer shows to the other side. It stands in for
/// the phone, which stays private until there is a deal.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DisplayName(String);

impl DisplayName {
    pub fn new(value: String) -> Result<Self, AlwaError> {
        let value = value.trim().to_string();

        if value.is_empty() {
            return Err(DomainError::InvalidValue("Name must not be empty".to_string()).into());
        }

        if value.chars().count() > MAX_LENGTH {
            return Err(DomainError::InvalidValue(format!(
                "Name must be {MAX_LENGTH} characters max"
            ))
            .into());
        }

        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<&DisplayName> for String {
    fn from(value: &DisplayName) -> Self {
        value.0.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn surrounding_whitespace_is_stripped_before_storing() {
        assert_eq!(
            DisplayName::new("  Kak Azad  ".to_string())
                .expect("name")
                .as_str(),
            "Kak Azad"
        );
    }

    #[test]
    fn a_name_of_only_whitespace_is_empty_not_valid() {
        assert!(DisplayName::new("   ".to_string()).is_err());
        assert!(DisplayName::new(String::new()).is_err());
    }

    #[test]
    fn the_limit_counts_characters_not_bytes() {
        assert!(
            DisplayName::new("ک".repeat(MAX_LENGTH)).is_ok(),
            "a Sorani name must not be rejected for its byte length"
        );
        assert!(DisplayName::new("ک".repeat(MAX_LENGTH + 1)).is_err());
    }
}
