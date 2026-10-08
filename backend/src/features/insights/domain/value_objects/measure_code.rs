use crate::{features::insights::domain::InsightError, shared::DomainError};

const MAX_LENGTH: usize = 40;

/// The name of one measure inside a reading, for example `level_pct`. The
/// backend does not know which codes exist; it only keeps them in one form
/// so the app can look a measure up by its code.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct MeasureCode(String);

impl MeasureCode {
    pub fn new(value: String) -> Result<Self, InsightError> {
        let well_formed = !value.is_empty()
            && value.len() <= MAX_LENGTH
            && value.chars().all(|character| {
                character.is_ascii_lowercase() || character.is_ascii_digit() || character == '_'
            });

        if !well_formed {
            return Err(DomainError::InvalidValue(format!(
                "Measure code must be 1 to {MAX_LENGTH} lower-case letters, digits and underscores"
            ))
            .into());
        }

        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<&MeasureCode> for String {
    fn from(value: &MeasureCode) -> Self {
        value.0.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_lower_case_letters_digits_and_underscores() {
        assert!(MeasureCode::new("level_pct".to_string()).is_ok());
        assert!(MeasureCode::new("rain_30d_mm".to_string()).is_ok());
        assert!(MeasureCode::new("7".to_string()).is_ok());
    }

    #[test]
    fn rejects_capitals_hyphens_spaces_and_other_scripts() {
        assert!(MeasureCode::new("Level".to_string()).is_err());
        assert!(MeasureCode::new("level-pct".to_string()).is_err());
        assert!(MeasureCode::new("level pct".to_string()).is_err());
        assert!(MeasureCode::new("ئاست".to_string()).is_err());
    }

    #[test]
    fn the_length_is_one_to_forty() {
        assert!(MeasureCode::new(String::new()).is_err());
        assert!(MeasureCode::new("a".repeat(MAX_LENGTH)).is_ok());
        assert!(MeasureCode::new("a".repeat(MAX_LENGTH + 1)).is_err());
    }
}
