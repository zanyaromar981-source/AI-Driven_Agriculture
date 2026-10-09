use crate::{features::staff::domain::StaffError, shared::DomainError};

const MAX_LENGTH: usize = 200;

/// A sentence on what a role is for. Optional.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RoleDescription(String);

impl RoleDescription {
    pub fn new(value: String) -> Result<Self, StaffError> {
        let value = value.trim().to_string();

        if value.is_empty() {
            return Err(DomainError::InvalidValue(
                "Role description must not be empty".to_string(),
            )
            .into());
        }

        if value.chars().count() > MAX_LENGTH {
            return Err(DomainError::InvalidValue(format!(
                "Role description must be {MAX_LENGTH} characters max"
            ))
            .into());
        }

        Ok(Self(value))
    }

    /// A form sends an untouched description as an empty text, which means
    /// the same as none.
    pub fn optional(value: Option<String>) -> Result<Option<Self>, StaffError> {
        match value {
            Some(value) if !value.trim().is_empty() => Self::new(value).map(Some),
            _ => Ok(None),
        }
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<&RoleDescription> for String {
    fn from(value: &RoleDescription) -> Self {
        value.0.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_blank_or_missing_description_is_none() {
        assert!(RoleDescription::optional(None).expect("none").is_none());
        assert!(
            RoleDescription::optional(Some("  ".to_string()))
                .expect("blank")
                .is_none()
        );
    }

    #[test]
    fn a_given_description_is_kept_trimmed() {
        let description = RoleDescription::optional(Some(" Reads the dams ".to_string()))
            .expect("valid")
            .expect("some");

        assert_eq!(description.as_str(), "Reads the dams");
    }

    #[test]
    fn the_limit_counts_characters_not_bytes() {
        assert!(RoleDescription::new("ک".repeat(MAX_LENGTH)).is_ok());
        assert!(RoleDescription::new("ک".repeat(MAX_LENGTH + 1)).is_err());
        assert!(RoleDescription::optional(Some("ک".repeat(MAX_LENGTH + 1))).is_err());
    }
}
