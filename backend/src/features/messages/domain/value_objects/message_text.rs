use crate::{features::messages::domain::MessageError, shared::DomainError};

/// Text a person wrote: what the farmer sent, or what staff answered.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MessageText(String);

impl MessageText {
    pub const MAX_LENGTH: usize = 2_000;

    /// No character takes more than 4 bytes in UTF-8, so text longer than
    /// this is too long whatever it says. A reader can stop there.
    pub const MAX_BYTES: usize = Self::MAX_LENGTH * 4;

    /// The limit counts characters, not bytes: Sorani text is multi-byte.
    pub fn new(value: String) -> Result<Self, MessageError> {
        let value = value.trim().to_string();

        if value.is_empty() {
            return Err(
                DomainError::InvalidValue("Message text must not be empty".to_string()).into(),
            );
        }

        if value.chars().count() > Self::MAX_LENGTH {
            return Err(MessageError::TextTooLong(Self::MAX_LENGTH));
        }

        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<&MessageText> for String {
    fn from(value: &MessageText) -> Self {
        value.0.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn surrounding_whitespace_is_stripped() {
        let text = MessageText::new("  The canal is dry.\n".to_string()).expect("text");

        assert_eq!(text.as_str(), "The canal is dry.");
    }

    #[test]
    fn text_of_only_whitespace_is_empty_not_valid() {
        assert!(MessageText::new(" \n\t ".to_string()).is_err());
        assert!(MessageText::new(String::new()).is_err());
    }

    #[test]
    fn the_length_limit_is_inclusive() {
        assert!(MessageText::new("a".repeat(MessageText::MAX_LENGTH)).is_ok());
        assert!(matches!(
            MessageText::new("a".repeat(MessageText::MAX_LENGTH + 1)),
            Err(MessageError::TextTooLong(2_000))
        ));
    }

    #[test]
    fn the_limit_counts_characters_not_bytes() {
        // Each of these letters is two bytes in UTF-8.
        let sorani = "ک".repeat(MessageText::MAX_LENGTH);

        assert!(sorani.len() > MessageText::MAX_LENGTH);
        assert!(MessageText::new(sorani).is_ok());
    }
}
