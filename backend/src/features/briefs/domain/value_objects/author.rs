use crate::features::briefs::domain::BriefError;

use super::text;

const MAX_LENGTH: usize = 80;

/// Which tool or model wrote the brief, for example `codex-cli gpt-5`, so
/// a reader can tell a machine's text from a person's.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Author(String);

impl Author {
    pub fn new(value: String) -> Result<Self, BriefError> {
        Ok(Self(text::bounded(value, "Author", MAX_LENGTH)?))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<&Author> for String {
    fn from(value: &Author) -> Self {
        value.0.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_must_say_something() {
        assert!(Author::new(String::new()).is_err());
        assert!(Author::new("   ".to_string()).is_err());
        assert_eq!(
            Author::new(" codex-cli gpt-5 ".to_string())
                .expect("text")
                .as_str(),
            "codex-cli gpt-5"
        );
    }

    #[test]
    fn the_limit_counts_characters_not_bytes() {
        assert!(
            Author::new("ئ".repeat(MAX_LENGTH)).is_ok(),
            "Sorani text at the limit must not be rejected for its byte length"
        );
        assert!(Author::new("ئ".repeat(MAX_LENGTH + 1)).is_err());
        assert!(Author::new("a".repeat(MAX_LENGTH + 1)).is_err());
    }
}
