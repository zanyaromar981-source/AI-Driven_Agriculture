use crate::{features::jobs::domain::JobError, shared::DomainError};

const MAX_LENGTH: usize = 500;

/// What a job says about one of its runs: a short summary when it went
/// well, the reason when it did not.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RunMessage(String);

impl RunMessage {
    /// A message of only spaces says nothing, so it is no message.
    pub fn new(value: String) -> Result<Option<Self>, JobError> {
        let value = value.trim().to_string();

        if value.is_empty() {
            return Ok(None);
        }

        // Characters, not bytes: a Sorani message is multi-byte.
        if value.chars().count() > MAX_LENGTH {
            return Err(DomainError::InvalidValue(format!(
                "message must be {MAX_LENGTH} characters max"
            ))
            .into());
        }

        Ok(Some(Self(value)))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<&RunMessage> for String {
    fn from(value: &RunMessage) -> Self {
        value.0.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn surrounding_whitespace_is_stripped_before_storing() {
        let message = RunMessage::new("  312 fires stored  ".to_string())
            .expect("message")
            .expect("some");

        assert_eq!(message.as_str(), "312 fires stored");
    }

    #[test]
    fn an_empty_message_is_no_message() {
        assert_eq!(RunMessage::new(String::new()).expect("message"), None);
        assert_eq!(RunMessage::new("   ".to_string()).expect("message"), None);
    }

    #[test]
    fn the_limit_is_inclusive_and_counted_in_characters() {
        assert!(RunMessage::new("a".repeat(MAX_LENGTH)).is_ok());
        assert!(RunMessage::new("a".repeat(MAX_LENGTH + 1)).is_err());
        assert!(
            RunMessage::new("ڕ".repeat(MAX_LENGTH)).is_ok(),
            "500 Sorani letters are 1000 bytes and still fit"
        );
    }
}
