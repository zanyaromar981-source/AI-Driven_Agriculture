use crate::{features::alerts::domain::AlertError, shared::DomainError};

const MAX_LENGTH: usize = 500;

/// One sentence or two a farmer reads: what is happening, or what to do
/// about it. Never empty, because every alert says both.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AlertText(String);

impl AlertText {
    pub fn new(value: String) -> Result<Self, AlertError> {
        let value = value.trim().to_string();

        if value.is_empty() || value.chars().count() > MAX_LENGTH {
            return Err(DomainError::InvalidValue(format!(
                "Alert text must be 1 to {MAX_LENGTH} characters"
            ))
            .into());
        }

        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<&AlertText> for String {
    fn from(value: &AlertText) -> Self {
        value.0.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn surrounding_whitespace_is_stripped_before_storing() {
        assert_eq!(
            AlertText::new(" Frost -3 °C Sun night ".to_string())
                .expect("text")
                .as_str(),
            "Frost -3 °C Sun night"
        );
    }

    #[test]
    fn an_empty_text_is_refused() {
        assert!(AlertText::new(String::new()).is_err());
        assert!(AlertText::new("  ".to_string()).is_err());
    }

    #[test]
    fn the_limit_is_inclusive_and_counts_characters() {
        assert!(AlertText::new("س".repeat(MAX_LENGTH)).is_ok());
        assert!(AlertText::new("س".repeat(MAX_LENGTH + 1)).is_err());
    }
}
