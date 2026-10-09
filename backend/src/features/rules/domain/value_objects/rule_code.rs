use crate::{features::rules::domain::RuleError, shared::DomainError};

const MIN_LENGTH: usize = 2;
const MAX_LENGTH: usize = 60;

/// The fixed name of a rule, as the code that reads it spells it: lower-case
/// letters, digits and underscores, for example `frost_c`.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct RuleCode(String);

impl RuleCode {
    pub fn new(value: String) -> Result<Self, RuleError> {
        let well_formed = (MIN_LENGTH..=MAX_LENGTH).contains(&value.len())
            && value
                .bytes()
                .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_');

        if !well_formed {
            return Err(DomainError::InvalidValue(
                "A rule code is lower-case letters, digits and underscores".to_string(),
            )
            .into());
        }

        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<&RuleCode> for String {
    fn from(value: &RuleCode) -> Self {
        value.0.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_seeded_shapes_are_accepted() {
        assert!(RuleCode::new("frost_c".to_string()).is_ok());
        assert!(RuleCode::new("dust_pm10".to_string()).is_ok());
    }

    #[test]
    fn anything_that_could_not_be_a_code_is_refused() {
        for bad in ["", "a", "Frost_c", "frost c", "frost-c", "frost/c", "ڕێسا"] {
            assert!(RuleCode::new(bad.to_string()).is_err(), "{bad:?}");
        }
    }

    #[test]
    fn the_length_limit_is_inclusive() {
        assert!(RuleCode::new("a".repeat(MAX_LENGTH)).is_ok());
        assert!(RuleCode::new("a".repeat(MAX_LENGTH + 1)).is_err());
    }
}
