use crate::{features::insights::domain::InsightError, shared::DomainError};

const MAX_LENGTH: usize = 300;

/// One or two sentences that say what a reading means, in one language. The
/// data job writes it; the backend only keeps it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Summary(String);

impl Summary {
    pub fn new(value: String) -> Result<Self, InsightError> {
        let value = value.trim().to_string();

        if value.is_empty() {
            return Err(DomainError::InvalidValue("Summary must not be empty".to_string()).into());
        }

        if value.chars().count() > MAX_LENGTH {
            return Err(DomainError::InvalidValue(format!(
                "Summary must be {MAX_LENGTH} characters max"
            ))
            .into());
        }

        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<&Summary> for String {
    fn from(value: &Summary) -> Self {
        value.0.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn surrounding_whitespace_is_stripped_before_storing() {
        assert_eq!(
            Summary::new("  The dam is half full.  ".to_string())
                .expect("summary")
                .as_str(),
            "The dam is half full."
        );
    }

    #[test]
    fn a_summary_of_only_whitespace_is_empty_not_valid() {
        assert!(
            Summary::new("   ".to_string()).is_err(),
            "a job with nothing to say sends null, not blanks"
        );
    }

    #[test]
    fn the_limit_counts_characters_not_bytes() {
        assert!(
            Summary::new("ئ".repeat(MAX_LENGTH)).is_ok(),
            "a Sorani summary must not be rejected for its byte length"
        );
        assert!(Summary::new("ئ".repeat(MAX_LENGTH + 1)).is_err());
    }
}
