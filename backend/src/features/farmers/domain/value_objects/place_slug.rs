use crate::{features::farmers::domain::FarmerError, shared::DomainError};

const MAX_LENGTH: usize = 40;

/// A governorate, zone or sub-zone a farmer lives in, by its slug, for
/// example `chamchamal`. Only the form is checked here: the places
/// themselves belong to another feature.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PlaceSlug(String);

impl PlaceSlug {
    pub fn new(value: String) -> Result<Self, FarmerError> {
        let well_formed = !value.is_empty()
            && value.len() <= MAX_LENGTH
            && value
                .chars()
                .all(|character| character.is_ascii_lowercase() || character == '-');

        if !well_formed {
            return Err(DomainError::InvalidValue(format!(
                "A place slug must be 1 to {MAX_LENGTH} lower-case letters and hyphens"
            ))
            .into());
        }

        Ok(Self(value))
    }

    /// A form or a query sends an untouched field as an empty text, which
    /// means the same as none.
    pub fn optional(value: Option<String>) -> Result<Option<Self>, FarmerError> {
        match value {
            Some(value) if !value.is_empty() => Self::new(value).map(Some),
            _ => Ok(None),
        }
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<&PlaceSlug> for String {
    fn from(value: &PlaceSlug) -> Self {
        value.0.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_a_plain_and_a_hyphenated_slug() {
        assert!(PlaceSlug::new("sulaymaniyah".to_string()).is_ok());
        assert!(PlaceSlug::new("qadir-karam".to_string()).is_ok());
    }

    #[test]
    fn rejects_capitals_digits_spaces_and_other_scripts() {
        for bad in ["Chamchamal", "zone1", "qadir karam", "چەمچەماڵ", " kalar"] {
            assert!(PlaceSlug::new(bad.to_string()).is_err(), "{bad}");
        }

        assert!(PlaceSlug::new("a".repeat(MAX_LENGTH + 1)).is_err());
    }

    #[test]
    fn an_empty_or_missing_text_is_no_place() {
        assert_eq!(PlaceSlug::optional(None).expect("none"), None);
        assert_eq!(
            PlaceSlug::optional(Some(String::new())).expect("none"),
            None
        );
    }
}
