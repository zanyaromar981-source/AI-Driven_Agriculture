use crate::{features::staff::domain::StaffError, shared::DomainError};

const MAX_LENGTH: usize = 80;

/// The name other staff see next to an account.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StaffName(String);

impl StaffName {
    pub fn new(value: String) -> Result<Self, StaffError> {
        let value = value.trim().to_string();

        if value.is_empty() {
            return Err(
                DomainError::InvalidValue("Staff name must not be empty".to_string()).into(),
            );
        }

        if value.chars().count() > MAX_LENGTH {
            return Err(DomainError::InvalidValue(format!(
                "Staff name must be {MAX_LENGTH} characters max"
            ))
            .into());
        }

        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<&StaffName> for String {
    fn from(value: &StaffName) -> Self {
        value.0.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn surrounding_whitespace_is_stripped_before_storing() {
        assert_eq!(
            StaffName::new("  Hiwa K.  ".to_string())
                .expect("name")
                .as_str(),
            "Hiwa K."
        );
    }

    #[test]
    fn a_name_of_only_whitespace_is_empty_not_valid() {
        assert!(StaffName::new("   ".to_string()).is_err());
    }

    #[test]
    fn the_limit_counts_characters_not_bytes() {
        assert!(StaffName::new("ک".repeat(MAX_LENGTH)).is_ok());
        assert!(StaffName::new("ک".repeat(MAX_LENGTH + 1)).is_err());
    }
}
