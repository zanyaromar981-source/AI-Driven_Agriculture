use crate::{features::workers::domain::WorkerError, shared::DomainError};

const MAX_LENGTH: usize = 100;

/// What a farmer typed in the search box. It is looked for as it is,
/// anywhere in the name or the note on a card,
/// whatever the case.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WorkerSearch(String);

impl WorkerSearch {
    pub fn new(value: String) -> Result<Self, WorkerError> {
        let value = value.trim().to_string();

        if value.is_empty() {
            return Err(
                DomainError::InvalidValue("Search text must not be empty".to_string()).into(),
            );
        }

        if value.chars().count() > MAX_LENGTH {
            return Err(DomainError::InvalidValue(format!(
                "Search text must be {MAX_LENGTH} characters max"
            ))
            .into());
        }

        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// The text as a `LIKE` pattern that matches it anywhere, with `\` as
    /// the escape character (the one Postgres uses when none is named). `%`
    /// and `_` mean "anything" to `LIKE`; typed by a person they are just
    /// characters, so they are escaped.
    pub fn like_pattern(&self) -> String {
        let mut pattern = String::with_capacity(self.0.len() + 2);

        pattern.push('%');

        for character in self.0.chars() {
            if matches!(character, '%' | '_' | '\\') {
                pattern.push('\\');
            }

            pattern.push(character);
        }

        pattern.push('%');
        pattern
    }
}

impl From<&WorkerSearch> for String {
    fn from(value: &WorkerSearch) -> Self {
        value.0.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn search(value: &str) -> WorkerSearch {
        WorkerSearch::new(value.to_string()).expect("search text")
    }

    #[test]
    fn surrounding_whitespace_is_stripped() {
        assert_eq!(search("  canal ").as_str(), "canal");
    }

    #[test]
    fn an_empty_or_over_long_search_is_refused() {
        assert!(WorkerSearch::new("  ".to_string()).is_err());
        assert!(WorkerSearch::new("a".repeat(MAX_LENGTH)).is_ok());
        assert!(WorkerSearch::new("a".repeat(MAX_LENGTH + 1)).is_err());
    }

    #[test]
    fn the_pattern_matches_the_text_anywhere() {
        assert_eq!(search("canal").like_pattern(), "%canal%");
    }

    #[test]
    fn the_wildcards_of_like_are_escaped_so_they_match_themselves() {
        assert_eq!(search("100%").like_pattern(), "%100\\%%");
        assert_eq!(search("a_b").like_pattern(), "%a\\_b%");
        assert_eq!(search("a\\b").like_pattern(), "%a\\\\b%");
    }
}
