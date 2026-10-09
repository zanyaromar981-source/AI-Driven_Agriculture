use crate::{features::plans::domain::PlanError, shared::DomainError};

const MAX_LENGTH: usize = 300;

/// One sentence of an alert or a decision, in one language. The data job
/// writes it; the backend only keeps it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PlanText(String);

impl PlanText {
    pub fn new(value: String) -> Result<Self, PlanError> {
        let value = value.trim().to_string();

        if value.is_empty() || value.chars().count() > MAX_LENGTH {
            return Err(DomainError::InvalidValue(format!(
                "Plan text must be 1 to {MAX_LENGTH} characters"
            ))
            .into());
        }

        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<&PlanText> for String {
    fn from(value: &PlanText) -> Self {
        value.0.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn surrounding_whitespace_is_stripped_before_storing() {
        assert_eq!(
            PlanText::new("  Frost -3 °C Sun night ".to_string())
                .expect("text")
                .as_str(),
            "Frost -3 °C Sun night"
        );
    }

    #[test]
    fn a_sentence_of_only_whitespace_is_refused() {
        assert!(
            PlanText::new("   ".to_string()).is_err(),
            "the app would show an empty line with an icon next to it"
        );
        assert!(PlanText::new(String::new()).is_err());
    }

    #[test]
    fn the_limit_counts_characters_not_bytes() {
        assert!(
            PlanText::new("ئ".repeat(MAX_LENGTH)).is_ok(),
            "a Sorani sentence must not be rejected for its byte length"
        );
        assert!(PlanText::new("ئ".repeat(MAX_LENGTH + 1)).is_err());
    }
}
