use crate::{features::water::domain::WaterError, shared::DomainError};

const MAX_LENGTH: usize = 200;

/// One short remark, in one language, on a zone's place in the water plan.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Note(String);

impl Note {
    pub fn new(value: String) -> Result<Self, WaterError> {
        let value = value.trim().to_string();

        if value.is_empty() {
            return Err(DomainError::InvalidValue("A note must not be empty".to_string()).into());
        }

        if value.chars().count() > MAX_LENGTH {
            return Err(DomainError::InvalidValue(format!(
                "A note must be {MAX_LENGTH} characters max"
            ))
            .into());
        }

        Ok(Self(value))
    }

    /// A missing note and a blank one are the same thing: no note.
    pub fn optional(value: Option<String>) -> Result<Option<Self>, WaterError> {
        match value {
            Some(value) if !value.trim().is_empty() => Ok(Some(Self::new(value)?)),
            _ => Ok(None),
        }
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
    fn surrounding_whitespace_is_stripped_before_storing() {
        let note = Note::new("  Canal under repair  ".to_string()).expect("note");

        assert_eq!(note.as_str(), "Canal under repair");
    }

    #[test]
    fn the_limit_is_inclusive() {
        assert!(Note::new("a".repeat(MAX_LENGTH)).is_ok());
        assert!(Note::new("a".repeat(MAX_LENGTH + 1)).is_err());
    }

    #[test]
    fn the_limit_counts_characters_not_bytes() {
        let kurdish = "ک".repeat(MAX_LENGTH);

        assert!(kurdish.len() > MAX_LENGTH);
        assert!(
            Note::new(kurdish).is_ok(),
            "a Sorani note must not be rejected for its byte length"
        );
    }

    #[test]
    fn a_missing_or_blank_note_is_no_note() {
        assert_eq!(Note::optional(None).expect("none"), None);
        assert_eq!(
            Note::optional(Some("   ".to_string())).expect("blank"),
            None
        );
        assert!(Note::new("   ".to_string()).is_err());
    }

    #[test]
    fn a_given_note_is_still_checked_for_length() {
        assert!(Note::optional(Some("a".repeat(MAX_LENGTH + 1))).is_err());
        assert_eq!(
            Note::optional(Some("Canal under repair".to_string()))
                .expect("note")
                .expect("some")
                .as_str(),
            "Canal under repair"
        );
    }
}
