use crate::{features::staff::domain::StaffError, shared::DomainError};

const MAX_LENGTH: usize = 80;

/// What a staff member does at the Ministry, as other staff read it.
/// Optional.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct JobTitle(String);

impl JobTitle {
    pub fn new(value: String) -> Result<Self, StaffError> {
        let value = value.trim().to_string();

        if value.is_empty() {
            return Err(
                DomainError::InvalidValue("Job title must not be empty".to_string()).into(),
            );
        }

        if value.chars().count() > MAX_LENGTH {
            return Err(DomainError::InvalidValue(format!(
                "Job title must be {MAX_LENGTH} characters max"
            ))
            .into());
        }

        Ok(Self(value))
    }

    /// A form sends an untouched field as an empty text, which means the
    /// same as none.
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

impl From<&JobTitle> for String {
    fn from(value: &JobTitle) -> Self {
        value.0.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn surrounding_whitespace_is_stripped_before_storing() {
        assert_eq!(
            JobTitle::new("  Dam engineer ".to_string())
                .expect("title")
                .as_str(),
            "Dam engineer"
        );
    }

    #[test]
    fn the_limit_counts_characters_not_bytes() {
        assert!(JobTitle::new("ک".repeat(MAX_LENGTH)).is_ok());
        assert!(JobTitle::new("ک".repeat(MAX_LENGTH + 1)).is_err());
    }

    #[test]
    fn an_empty_or_missing_text_is_no_title() {
        assert_eq!(JobTitle::optional(None).expect("none"), None);
        assert_eq!(
            JobTitle::optional(Some("  ".to_string())).expect("none"),
            None
        );
    }
}
