use crate::{features::jobs::domain::JobError, shared::DomainError};

const MIN_LENGTH: usize = 2;
const MAX_LENGTH: usize = 40;

/// The fixed name of a data job, as the job itself spells it when it
/// reports: lower-case letters and underscores, for example `farm_analysis`.
#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct JobCode(String);

impl JobCode {
    pub fn new(value: String) -> Result<Self, JobError> {
        let well_formed = (MIN_LENGTH..=MAX_LENGTH).contains(&value.len())
            && value
                .bytes()
                .all(|byte| byte.is_ascii_lowercase() || byte == b'_');

        if !well_formed {
            return Err(DomainError::InvalidValue(
                "A job code is lower-case letters and underscores".to_string(),
            )
            .into());
        }

        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<&JobCode> for String {
    fn from(value: &JobCode) -> Self {
        value.0.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_seeded_shapes_are_accepted() {
        assert!(JobCode::new("fires".to_string()).is_ok());
        assert!(JobCode::new("farm_analysis".to_string()).is_ok());
    }

    #[test]
    fn anything_that_could_not_be_a_code_is_refused() {
        for bad in ["", "a", "Fires", "fire s", "fires-2", "fires/..", "fires2"] {
            assert!(JobCode::new(bad.to_string()).is_err(), "{bad:?}");
        }
    }

    #[test]
    fn the_length_limit_is_inclusive() {
        assert!(JobCode::new("a".repeat(MAX_LENGTH)).is_ok());
        assert!(JobCode::new("a".repeat(MAX_LENGTH + 1)).is_err());
    }
}
