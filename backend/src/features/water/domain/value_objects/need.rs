use crate::{features::water::domain::WaterError, shared::DomainError};

/// How badly a zone needs water, from 0 (not at all) to 100 (most).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Need(f64);

impl Need {
    pub fn new(value: f64) -> Result<Self, WaterError> {
        // A NaN fails the range check too, so it never reaches the ranking.
        if !(0.0..=100.0).contains(&value) {
            return Err(
                DomainError::InvalidValue("need must be between 0 and 100".to_string()).into(),
            );
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
        assert_eq!(Need::new(0.0).expect("none").value(), 0.0);
        assert_eq!(Need::new(100.0).expect("most").value(), 100.0);
        assert_eq!(Need::new(63.5).expect("part").value(), 63.5);
    }

    #[test]
    fn a_value_off_the_scale_is_refused() {
        assert!(Need::new(-1.0).is_err());
        assert!(Need::new(100.5).is_err());
        assert!(Need::new(f64::NAN).is_err());
    }
}
