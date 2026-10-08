use crate::{features::outlooks::domain::OutlookError, shared::DomainError};

/// How sure the method is of an outlook, from 0 to 100.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Confidence(f64);

impl Confidence {
    pub fn new(value: f64) -> Result<Self, OutlookError> {
        // A NaN fails the range check too, so it never reaches the database.
        if !(0.0..=100.0).contains(&value) {
            return Err(DomainError::InvalidValue(
                "confidence_pct must be between 0 and 100".to_string(),
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
        assert_eq!(Confidence::new(0.0).expect("none").value(), 0.0);
        assert_eq!(Confidence::new(100.0).expect("sure").value(), 100.0);
        assert_eq!(Confidence::new(72.5).expect("part").value(), 72.5);
    }

    #[test]
    fn a_value_off_the_scale_is_refused() {
        assert!(Confidence::new(-1.0).is_err());
        assert!(Confidence::new(100.5).is_err());
        assert!(Confidence::new(f64::NAN).is_err());
    }
}
