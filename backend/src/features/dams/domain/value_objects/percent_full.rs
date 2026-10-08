use crate::{features::dams::domain::DamError, shared::DomainError};

/// How full a reservoir is, from 0 (empty) to 100 (at capacity).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PercentFull(f64);

impl PercentFull {
    pub fn new(value: f64) -> Result<Self, DamError> {
        // A NaN fails the range check too, so it never reaches the database.
        if !(0.0..=100.0).contains(&value) {
            return Err(DomainError::InvalidValue(
                "pct_full must be between 0 and 100".to_string(),
            )
            .into());
        }

        Ok(Self(value))
    }

    pub fn value(&self) -> f64 {
        self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn both_ends_of_the_scale_are_allowed() {
        assert_eq!(PercentFull::new(0.0).expect("empty").value(), 0.0);
        assert_eq!(PercentFull::new(100.0).expect("full").value(), 100.0);
        assert_eq!(PercentFull::new(47.3).expect("part").value(), 47.3);
    }

    #[test]
    fn a_value_off_the_scale_is_refused() {
        assert!(PercentFull::new(-0.1).is_err());
        assert!(PercentFull::new(100.1).is_err());
    }

    #[test]
    fn a_value_that_is_not_a_number_is_refused() {
        assert!(PercentFull::new(f64::NAN).is_err());
        assert!(PercentFull::new(f64::INFINITY).is_err());
    }
}
