use crate::{features::water::domain::WaterError, shared::DomainError};

const MAX_LENGTH: usize = 40;

/// The dam a zone is to be supplied from, for example `dukan`. Only its form
/// is checked here: the dams themselves belong to another slice.
#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct DamSlug(String);

impl DamSlug {
    pub fn new(value: String) -> Result<Self, WaterError> {
        let well_formed = !value.is_empty()
            && value.len() <= MAX_LENGTH
            && value
                .bytes()
                .all(|byte| byte.is_ascii_lowercase() || byte == b'-');

        if !well_formed {
            return Err(DomainError::InvalidValue(format!(
                "A dam slug is 1 to {MAX_LENGTH} lower-case letters and hyphens"
            ))
            .into());
        }

        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<&DamSlug> for String {
    fn from(value: &DamSlug) -> Self {
        value.0.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lower_case_letters_and_hyphens_are_a_slug() {
        assert_eq!(
            DamSlug::new("dukan".to_string()).expect("slug").as_str(),
            "dukan"
        );
        assert!(DamSlug::new("upper-dam".to_string()).is_ok());
    }

    #[test]
    fn anything_else_is_not_a_slug() {
        for bad in [
            "",
            "Dukan",
            "dukan 1",
            "dukan_1",
            "dam2",
            "دووکان",
            " dukan",
        ] {
            assert!(
                DamSlug::new(bad.to_string()).is_err(),
                "{bad:?} was accepted"
            );
        }
    }

    #[test]
    fn the_length_limit_is_inclusive() {
        assert!(DamSlug::new("a".repeat(MAX_LENGTH)).is_ok());
        assert!(DamSlug::new("a".repeat(MAX_LENGTH + 1)).is_err());
    }
}
