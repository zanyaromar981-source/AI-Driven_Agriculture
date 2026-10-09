use crate::{features::doctor::domain::DoctorError, shared::DomainError};

/// What the farmer typed, in their own words.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Question(String);

impl Question {
    pub const MAX_LENGTH: usize = 1_000;

    /// No character takes more than 4 bytes in UTF-8, so text longer than
    /// this is too long whatever it says. A reader can stop there.
    pub const MAX_BYTES: usize = Self::MAX_LENGTH * 4;

    pub fn new(value: String) -> Result<Self, DoctorError> {
        let value = value.trim().to_string();

        if value.is_empty() {
            return Err(DomainError::InvalidValue("Question must not be empty".to_string()).into());
        }

        if value.chars().count() > Self::MAX_LENGTH {
            return Err(DoctorError::QuestionTooLong(Self::MAX_LENGTH));
        }

        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn surrounding_whitespace_is_stripped() {
        let question = Question::new("  Why are the leaves yellow?\n".to_string()).expect("q");

        assert_eq!(question.as_str(), "Why are the leaves yellow?");
    }

    #[test]
    fn a_question_of_only_whitespace_is_empty_not_valid() {
        assert!(Question::new(" \n\t ".to_string()).is_err());
        assert!(Question::new(String::new()).is_err());
    }

    #[test]
    fn the_limit_is_inclusive() {
        assert!(Question::new("a".repeat(Question::MAX_LENGTH)).is_ok());
        assert!(matches!(
            Question::new("a".repeat(Question::MAX_LENGTH + 1)),
            Err(DoctorError::QuestionTooLong(Question::MAX_LENGTH))
        ));
    }

    #[test]
    fn the_limit_counts_characters_not_bytes() {
        let sorani = "گەڵا".repeat(Question::MAX_LENGTH / 4);

        assert!(
            sorani.len() > Question::MAX_LENGTH,
            "this question is over the limit in bytes, which is the point"
        );
        assert!(
            Question::new(sorani).is_ok(),
            "a Sorani question must not be refused for its byte length"
        );
    }

    #[test]
    fn text_over_the_byte_bound_is_always_over_the_character_limit() {
        let widest = "😀".repeat(Question::MAX_LENGTH + 1);

        assert!(widest.len() > Question::MAX_BYTES);
        assert!(
            Question::new(widest).is_err(),
            "a reader that stops at MAX_BYTES must never stop on a valid question"
        );
        assert_eq!(
            "😀".repeat(Question::MAX_LENGTH).len(),
            Question::MAX_BYTES,
            "the longest valid question fits exactly"
        );
    }
}
