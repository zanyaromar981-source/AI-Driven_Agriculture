use crate::{features::farms::domain::FarmError, shared::DomainError};

const MAX_LENGTH: usize = 100;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FarmName(String);

impl FarmName {
    pub fn new(value: String) -> Result<Self, FarmError> {
        let value = value.trim().to_string();

        if value.is_empty() {
            return Err(
                DomainError::InvalidValue("Farm name must not be empty".to_string()).into(),
            );
        }

        if value.chars().count() > MAX_LENGTH {
            return Err(DomainError::InvalidValue(format!(
                "Farm name must be {MAX_LENGTH} characters max"
            ))
            .into());
        }

        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<FarmName> for String {
    fn from(value: FarmName) -> Self {
        value.0
    }
}

impl From<&FarmName> for String {
    fn from(value: &FarmName) -> Self {
        value.0.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn surrounding_whitespace_is_stripped_before_storing() {
        let name = FarmName::new("  Upper field  ".to_string()).expect("name");

        assert_eq!(name.as_str(), "Upper field");
    }

    #[test]
    fn a_name_of_only_whitespace_is_empty_not_valid() {
        assert!(
            FarmName::new("   ".to_string()).is_err(),
            "trimming must happen before the emptiness check, not after"
        );
        assert!(FarmName::new(String::new()).is_err());
    }

    #[test]
    fn the_limit_is_inclusive() {
        assert!(FarmName::new("a".repeat(MAX_LENGTH)).is_ok());
        assert!(FarmName::new("a".repeat(MAX_LENGTH + 1)).is_err());
    }

    #[test]
    fn the_limit_counts_characters_not_bytes() {
        let kurdish = "ک".repeat(MAX_LENGTH);

        assert!(
            kurdish.len() > MAX_LENGTH,
            "this string is longer than the limit in bytes, which is the point"
        );
        assert!(
            FarmName::new(kurdish).is_ok(),
            "a Sorani name must not be rejected for its byte length"
        );
    }
}
