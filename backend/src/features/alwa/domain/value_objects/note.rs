use crate::{features::alwa::domain::AlwaError, shared::DomainError};

const MAX_LENGTH: usize = 200;

/// A few words the seller adds to a listing, in their own language.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Note(String);

impl Note {
    pub fn new(value: String) -> Result<Self, AlwaError> {
        let value = value.trim().to_string();

        if value.is_empty() {
            return Err(DomainError::InvalidValue(
                "Note must not be empty, leave it out instead".to_string(),
            )
            .into());
        }

        if value.chars().count() > MAX_LENGTH {
            return Err(DomainError::InvalidValue(format!(
                "Note must be {MAX_LENGTH} characters max"
            ))
            .into());
        }

        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<&Note> for String {
    fn from(value: &Note) -> Self {
        value.0.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_limit_is_inclusive_and_counts_characters() {
        assert!(Note::new("ک".repeat(MAX_LENGTH)).is_ok());
        assert!(Note::new("ک".repeat(MAX_LENGTH + 1)).is_err());
    }

    #[test]
    fn a_blank_note_is_refused() {
        assert!(Note::new("  ".to_string()).is_err());
    }

    #[test]
    fn surrounding_whitespace_is_stripped() {
        assert_eq!(
            Note::new(" picked today ".to_string())
                .expect("note")
                .as_str(),
            "picked today"
        );
    }
}
