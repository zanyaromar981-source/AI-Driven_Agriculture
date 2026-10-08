use crate::{features::fires::domain::FireError, shared::DomainError};

const MAX_LENGTH: usize = 100;

/// The key the data job gives a detection. Pushing the same key again
/// updates that fire instead of making a second one.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExternalId(String);

impl ExternalId {
    pub fn new(value: String) -> Result<Self, FireError> {
        if value.is_empty() || value.chars().count() > MAX_LENGTH {
            return Err(DomainError::InvalidValue(format!(
                "External id must be 1 to {MAX_LENGTH} characters"
            ))
            .into());
        }

        // The id travels in the path, so a space or a control character in
        // it would make two spellings of one key.
        if !value.chars().all(|character| character.is_ascii_graphic()) {
            return Err(DomainError::InvalidValue(
                "External id must be printable ASCII without spaces".to_string(),
            )
            .into());
        }

        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<&ExternalId> for String {
    fn from(value: &ExternalId) -> Self {
        value.0.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_the_kind_of_key_a_job_builds() {
        assert!(ExternalId::new("firms:VIIRS_SNPP:2026-10-08:36.031,44.602".to_string()).is_ok());
    }

    #[test]
    fn the_limit_is_inclusive() {
        assert!(ExternalId::new("k".repeat(MAX_LENGTH)).is_ok());
        assert!(ExternalId::new("k".repeat(MAX_LENGTH + 1)).is_err());
    }

    #[test]
    fn rejects_an_empty_key() {
        assert!(ExternalId::new(String::new()).is_err());
    }

    #[test]
    fn rejects_spaces_control_characters_and_non_ascii() {
        assert!(ExternalId::new("two words".to_string()).is_err());
        assert!(ExternalId::new("line\nbreak".to_string()).is_err());
        assert!(ExternalId::new("ئاگر-1".to_string()).is_err());
    }
}
