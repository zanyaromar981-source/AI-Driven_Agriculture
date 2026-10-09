use crate::{features::crops::domain::CropError, shared::DomainError};

const MAX_CHARS: usize = 60;

/// What a crop is called in one language, as people read it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CropName(String);

impl CropName {
    pub fn new(value: String) -> Result<Self, CropError> {
        let value = value.trim().to_string();

        if value.is_empty() {
            return Err(
                DomainError::InvalidValue("A crop name cannot be empty".to_string()).into(),
            );
        }

        // Characters, not bytes: a Sorani name is multi-byte.
        if value.chars().count() > MAX_CHARS {
            return Err(DomainError::InvalidValue(format!(
                "A crop name is at most {MAX_CHARS} characters"
            ))
            .into());
        }

        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<&CropName> for String {
    fn from(value: &CropName) -> Self {
        value.0.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_name_is_trimmed() {
        assert_eq!(
            CropName::new("  Wheat ".to_string())
                .expect("name")
                .as_str(),
            "Wheat"
        );
    }

    #[test]
    fn an_empty_name_is_refused() {
        assert!(CropName::new(String::new()).is_err());
        assert!(CropName::new("   ".to_string()).is_err());
    }

    #[test]
    fn the_limit_counts_characters_not_bytes() {
        assert!(CropName::new("گ".repeat(MAX_CHARS)).is_ok());
        assert!(CropName::new("گ".repeat(MAX_CHARS + 1)).is_err());
    }
}
