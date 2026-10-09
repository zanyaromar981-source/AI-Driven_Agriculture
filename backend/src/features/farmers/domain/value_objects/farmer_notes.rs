use crate::{features::farmers::domain::FarmerError, shared::DomainError};

const MAX_LENGTH: usize = 1000;

/// What Ministry staff write down about a farmer. Staff only: it never
/// travels on a route the farmer app reaches.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FarmerNotes(String);

impl FarmerNotes {
    pub fn new(value: String) -> Result<Self, FarmerError> {
        let value = value.trim().to_string();

        if value.is_empty() {
            return Err(DomainError::InvalidValue("Notes must not be empty".to_string()).into());
        }

        if value.chars().count() > MAX_LENGTH {
            return Err(DomainError::InvalidValue(format!(
                "Notes must be {MAX_LENGTH} characters max"
            ))
            .into());
        }

        Ok(Self(value))
    }

    /// A form sends an untouched field as an empty text, which means the
    /// same as none.
    pub fn optional(value: Option<String>) -> Result<Option<Self>, FarmerError> {
        match value {
            Some(value) if !value.trim().is_empty() => Self::new(value).map(Some),
            _ => Ok(None),
        }
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<&FarmerNotes> for String {
    fn from(value: &FarmerNotes) -> Self {
        value.0.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_limit_counts_characters_not_bytes() {
        assert!(FarmerNotes::new("ک".repeat(MAX_LENGTH)).is_ok());
        assert!(FarmerNotes::new("ک".repeat(MAX_LENGTH + 1)).is_err());
    }

    #[test]
    fn an_empty_or_missing_text_is_no_notes() {
        assert_eq!(FarmerNotes::optional(None).expect("none"), None);
        assert_eq!(
            FarmerNotes::optional(Some(" ".to_string())).expect("none"),
            None
        );
    }
}
