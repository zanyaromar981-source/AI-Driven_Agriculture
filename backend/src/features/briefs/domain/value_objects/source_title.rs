use crate::features::briefs::domain::BriefError;

use super::text;

const MAX_LENGTH: usize = 200;

/// The name of a page the nightly job read, as it is shown to the reader.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SourceTitle(String);

impl SourceTitle {
    pub fn new(value: String) -> Result<Self, BriefError> {
        Ok(Self(text::bounded(value, "Source title", MAX_LENGTH)?))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<&SourceTitle> for String {
    fn from(value: &SourceTitle) -> Self {
        value.0.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_must_say_something() {
        assert!(SourceTitle::new(String::new()).is_err());
        assert!(SourceTitle::new("   ".to_string()).is_err());
        assert_eq!(
            SourceTitle::new(" FAO crop calendar ".to_string())
                .expect("text")
                .as_str(),
            "FAO crop calendar"
        );
    }

    #[test]
    fn the_limit_counts_characters_not_bytes() {
        assert!(
            SourceTitle::new("ئ".repeat(MAX_LENGTH)).is_ok(),
            "Sorani text at the limit must not be rejected for its byte length"
        );
        assert!(SourceTitle::new("ئ".repeat(MAX_LENGTH + 1)).is_err());
        assert!(SourceTitle::new("a".repeat(MAX_LENGTH + 1)).is_err());
    }
}
