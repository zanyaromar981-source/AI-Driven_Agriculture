use crate::{features::farmers::domain::FarmerError, shared::DomainError};

const MIN_LENGTH: usize = 3;
const MAX_LENGTH: usize = 300;

/// What a support letter is for, as the staff member wrote it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LetterPurpose(String);

impl LetterPurpose {
    pub fn new(value: String) -> Result<Self, FarmerError> {
        let value = value.trim().to_string();
        let length = value.chars().count();

        if !(MIN_LENGTH..=MAX_LENGTH).contains(&length) {
            return Err(DomainError::InvalidValue(format!(
                "Letter purpose must be {MIN_LENGTH} to {MAX_LENGTH} characters"
            ))
            .into());
        }

        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<&LetterPurpose> for String {
    fn from(value: &LetterPurpose) -> Self {
        value.0.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_purpose_needs_three_to_three_hundred_characters() {
        assert!(LetterPurpose::new("ab".to_string()).is_err());
        assert!(LetterPurpose::new("abc".to_string()).is_ok());
        assert!(LetterPurpose::new("a".repeat(MAX_LENGTH)).is_ok());
        assert!(LetterPurpose::new("a".repeat(MAX_LENGTH + 1)).is_err());
    }

    #[test]
    fn the_limits_count_characters_after_trimming() {
        assert!(LetterPurpose::new("  ab  ".to_string()).is_err());
        assert!(LetterPurpose::new("ک".repeat(MAX_LENGTH)).is_ok());
    }
}
