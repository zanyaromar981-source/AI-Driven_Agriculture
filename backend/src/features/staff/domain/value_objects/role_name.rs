use crate::{features::staff::domain::StaffError, shared::DomainError};

const MAX_LENGTH: usize = 60;

/// What a role is called in the role editor. Unique among roles.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RoleName(String);

impl RoleName {
    pub fn new(value: String) -> Result<Self, StaffError> {
        let value = value.trim().to_string();

        if value.is_empty() {
            return Err(
                DomainError::InvalidValue("Role name must not be empty".to_string()).into(),
            );
        }

        if value.chars().count() > MAX_LENGTH {
            return Err(DomainError::InvalidValue(format!(
                "Role name must be {MAX_LENGTH} characters max"
            ))
            .into());
        }

        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<&RoleName> for String {
    fn from(value: &RoleName) -> Self {
        value.0.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn surrounding_whitespace_is_stripped_before_storing() {
        assert_eq!(
            RoleName::new("  Dam officer ".to_string())
                .expect("name")
                .as_str(),
            "Dam officer"
        );
    }

    #[test]
    fn a_name_of_only_whitespace_is_empty_not_valid() {
        assert!(RoleName::new("   ".to_string()).is_err());
    }

    #[test]
    fn the_limit_counts_characters_not_bytes() {
        assert!(RoleName::new("ک".repeat(MAX_LENGTH)).is_ok());
        assert!(RoleName::new("ک".repeat(MAX_LENGTH + 1)).is_err());
    }
}
