use crate::{features::outlooks::domain::OutlookError, shared::DomainError};

const MAX_LENGTH: usize = 200;

/// One short sentence, in one language, on why a zone has its outlook.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Reason(String);

impl Reason {
    pub fn new(value: String) -> Result<Self, OutlookError> {
        let value = value.trim().to_string();

        if value.is_empty() {
            return Err(DomainError::InvalidValue("A reason must not be empty".to_string()).into());
        }

        if value.chars().count() > MAX_LENGTH {
            return Err(DomainError::InvalidValue(format!(
                "A reason must be {MAX_LENGTH} characters max"
            ))
            .into());
        }

        Ok(Self(value))
    }

    /// A missing reason and a blank one are the same thing: no reason.
    pub fn optional(value: Option<String>) -> Result<Option<Self>, OutlookError> {
        match value {
            Some(value) if !value.trim().is_empty() => Ok(Some(Self::new(value)?)),
            _ => Ok(None),
        }
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<&Reason> for String {
    fn from(value: &Reason) -> Self {
        value.0.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn surrounding_whitespace_is_stripped_before_storing() {
        let reason = Reason::new("  Rain 40% below normal  ".to_string()).expect("reason");

        assert_eq!(reason.as_str(), "Rain 40% below normal");
    }

    #[test]
    fn the_limit_is_inclusive() {
        assert!(Reason::new("a".repeat(MAX_LENGTH)).is_ok());
        assert!(Reason::new("a".repeat(MAX_LENGTH + 1)).is_err());
    }

    #[test]
    fn the_limit_counts_characters_not_bytes() {
        let kurdish = "ک".repeat(MAX_LENGTH);

        assert!(kurdish.len() > MAX_LENGTH);
        assert!(
            Reason::new(kurdish).is_ok(),
            "a Sorani reason must not be rejected for its byte length"
        );
    }

    #[test]
    fn a_missing_or_blank_reason_is_no_reason() {
        assert_eq!(Reason::optional(None).expect("none"), None);
        assert_eq!(
            Reason::optional(Some("   ".to_string())).expect("blank"),
            None
        );
        assert!(Reason::new("   ".to_string()).is_err());
    }

    #[test]
    fn a_given_reason_is_still_checked_for_length() {
        assert!(Reason::optional(Some("a".repeat(MAX_LENGTH + 1))).is_err());
        assert_eq!(
            Reason::optional(Some("Dry autumn".to_string()))
                .expect("reason")
                .expect("some")
                .as_str(),
            "Dry autumn"
        );
    }
}
