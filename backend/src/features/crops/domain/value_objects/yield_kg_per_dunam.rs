use crate::{features::crops::domain::CropError, shared::DomainError};

const MAX: f64 = 100_000.0;

/// What one dunam of the crop gives in a normal year, in kilograms. Staff
/// enter it from their own records; where there is none the crop has no
/// yield at all rather than a guessed one.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct YieldKgPerDunam(f64);

impl YieldKgPerDunam {
    pub fn new(value: f64) -> Result<Self, CropError> {
        if !value.is_finite() || value <= 0.0 || value > MAX {
            return Err(DomainError::InvalidValue(format!(
                "The yield must be above 0 and at most {MAX} kg per dunam"
            ))
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
    fn a_positive_yield_is_accepted() {
        assert_eq!(YieldKgPerDunam::new(750.0).expect("yield").value(), 750.0);
        assert!(YieldKgPerDunam::new(MAX).is_ok());
    }

    #[test]
    fn zero_negative_huge_and_not_a_number_are_refused() {
        for bad in [0.0, -1.0, MAX + 1.0, f64::NAN, f64::INFINITY] {
            assert!(YieldKgPerDunam::new(bad).is_err(), "{bad}");
        }
    }
}
