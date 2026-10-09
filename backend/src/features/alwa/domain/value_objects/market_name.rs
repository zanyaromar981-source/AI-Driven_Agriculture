use crate::{features::alwa::domain::AlwaError, shared::DomainError};

const MAX_LENGTH: usize = 80;

/// What an alwa is called in one language, as staff type it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MarketName(String);

impl MarketName {
    pub fn new(value: String) -> Result<Self, AlwaError> {
        let value = value.trim().to_string();

        if value.is_empty() || value.chars().count() > MAX_LENGTH {
            return Err(DomainError::InvalidValue(format!(
                "Market name must be 1 to {MAX_LENGTH} characters"
            ))
            .into());
        }

        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<&MarketName> for String {
    fn from(value: &MarketName) -> Self {
        value.0.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_limit_is_inclusive_and_counts_characters() {
        assert!(MarketName::new("ک".repeat(MAX_LENGTH)).is_ok());
        assert!(MarketName::new("ک".repeat(MAX_LENGTH + 1)).is_err());
    }

    #[test]
    fn a_blank_name_is_refused() {
        assert!(MarketName::new("  ".to_string()).is_err());
    }

    #[test]
    fn surrounding_whitespace_is_stripped() {
        assert_eq!(
            MarketName::new(" Halabja ".to_string())
                .expect("name")
                .as_str(),
            "Halabja"
        );
    }
}
