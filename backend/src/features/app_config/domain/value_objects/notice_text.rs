use crate::{features::app_config::domain::AppConfigError, shared::DomainError};

const MAX_LENGTH: usize = 500;

/// A short text the app shows the farmer: why to update, why the service is
/// down, an announcement.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NoticeText(String);

impl NoticeText {
    /// The limit counts characters, not bytes: Sorani text is multi-byte.
    pub fn new(value: String) -> Result<Self, AppConfigError> {
        let value = value.trim().to_string();

        if value.is_empty() {
            return Err(DomainError::InvalidValue("A text must not be empty".to_string()).into());
        }

        if value.chars().count() > MAX_LENGTH {
            return Err(DomainError::InvalidValue(format!(
                "A text must be {MAX_LENGTH} characters max"
            ))
            .into());
        }

        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<&NoticeText> for String {
    fn from(value: &NoticeText) -> Self {
        value.0.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn surrounding_whitespace_is_stripped() {
        let text = NoticeText::new("  Please update. ".to_string()).expect("text");

        assert_eq!(text.as_str(), "Please update.");
    }

    #[test]
    fn an_empty_text_is_refused() {
        assert!(NoticeText::new("  ".to_string()).is_err());
    }

    #[test]
    fn the_limit_is_inclusive_and_counts_characters() {
        assert!(NoticeText::new("ک".repeat(MAX_LENGTH)).is_ok());
        assert!(NoticeText::new("ک".repeat(MAX_LENGTH + 1)).is_err());
    }
}
