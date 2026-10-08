use crate::{features::farmers::domain::FarmerError, shared::DomainError};

const MAX_LENGTH: usize = 60;

/// The name a farmer chooses to show to buyers. It is optional: the phone is
/// the account.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FarmerName(String);

impl FarmerName {
    pub fn new(value: String) -> Result<Self, FarmerError> {
        let value = value.trim().to_string();

        if value.is_empty() {
            return Err(
                DomainError::InvalidValue("Farmer name must not be empty".to_string()).into(),
            );
        }

        if value.chars().count() > MAX_LENGTH {
            return Err(DomainError::InvalidValue(format!(
                "Farmer name must be {MAX_LENGTH} characters max"
            ))
            .into());
        }

        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<&FarmerName> for String {
    fn from(value: &FarmerName) -> Self {
        value.0.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn surrounding_whitespace_is_stripped_before_storing() {
        assert_eq!(
            FarmerName::new("  Hiwa K.  ".to_string())
                .expect("name")
                .as_str(),
            "Hiwa K."
        );
    }

    #[test]
    fn a_name_of_only_whitespace_is_empty_not_valid() {
        assert!(FarmerName::new("   ".to_string()).is_err());
    }

    #[test]
    fn the_limit_counts_characters_not_bytes() {
        assert!(FarmerName::new("ک".repeat(MAX_LENGTH)).is_ok());
        assert!(FarmerName::new("ک".repeat(MAX_LENGTH + 1)).is_err());
    }
}
