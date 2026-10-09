use crate::{features::farmers::domain::FarmerError, shared::DomainError};

const MAX_LENGTH: usize = 80;

/// The village a farmer lives in, as free text: villages are not in the
/// zones tables.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Village(String);

impl Village {
    pub fn new(value: String) -> Result<Self, FarmerError> {
        let value = value.trim().to_string();

        if value.is_empty() {
            return Err(DomainError::InvalidValue("Village must not be empty".to_string()).into());
        }

        if value.chars().count() > MAX_LENGTH {
            return Err(DomainError::InvalidValue(format!(
                "Village must be {MAX_LENGTH} characters max"
            ))
            .into());
        }

        Ok(Self(value))
    }

    /// A form sends an untouched field as an empty text, which means the
    /// same as none.
    pub fn optional(value: Option<String>) -> Result<Option<Self>, FarmerError> {
        match value {
            Some(value) if !value.trim().is_empty() => Self::new(value).map(Some),
            _ => Ok(None),
        }
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<&Village> for String {
    fn from(value: &Village) -> Self {
        value.0.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn surrounding_whitespace_is_stripped_before_storing() {
        assert_eq!(
            Village::new("  Qadir Karam ".to_string())
                .expect("village")
                .as_str(),
            "Qadir Karam"
        );
    }

    #[test]
    fn the_limit_counts_characters_not_bytes() {
        assert!(Village::new("ک".repeat(MAX_LENGTH)).is_ok());
        assert!(Village::new("ک".repeat(MAX_LENGTH + 1)).is_err());
    }

    #[test]
    fn an_empty_or_missing_text_is_no_village() {
        assert_eq!(Village::optional(None).expect("none"), None);
        assert_eq!(
            Village::optional(Some("  ".to_string())).expect("none"),
            None
        );
        assert!(
            Village::optional(Some("Sangaw".to_string()))
                .expect("some")
                .is_some()
        );
    }
}
