use crate::{features::farmers::domain::FarmerError, shared::DomainError};

const MAX_LENGTH: usize = 80;

/// A piece of a name or a phone that staff type to find a farmer.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FarmerSearch(String);

impl FarmerSearch {
    /// An empty text is no search. The length is bounded before the text
    /// reaches the database.
    pub fn optional(value: Option<String>) -> Result<Option<Self>, FarmerError> {
        let Some(value) = value.map(|value| value.trim().to_string()) else {
            return Ok(None);
        };

        if value.is_empty() {
            return Ok(None);
        }

        if value.chars().count() > MAX_LENGTH {
            return Err(DomainError::InvalidValue(format!(
                "Search text must be {MAX_LENGTH} characters max"
            ))
            .into());
        }

        Ok(Some(Self(value)))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_empty_or_missing_text_is_no_search() {
        assert_eq!(FarmerSearch::optional(None).expect("none"), None);
        assert_eq!(
            FarmerSearch::optional(Some("  ".to_string())).expect("none"),
            None
        );
    }

    #[test]
    fn the_text_is_trimmed_and_bounded() {
        assert_eq!(
            FarmerSearch::optional(Some(" hiwa ".to_string()))
                .expect("search")
                .expect("some")
                .as_str(),
            "hiwa"
        );
        assert!(FarmerSearch::optional(Some("ک".repeat(MAX_LENGTH))).is_ok());
        assert!(FarmerSearch::optional(Some("ک".repeat(MAX_LENGTH + 1))).is_err());
    }
}
