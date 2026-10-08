use crate::features::zones::domain::ZoneError;

/// Rain this month as a percentage of the long-run normal for the same
/// month: 100 is a normal month, 0 is no rain at all.
#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
pub struct RainPctOfNormal(f64);

impl RainPctOfNormal {
    pub const MIN: f64 = 0.0;
    pub const MAX: f64 = 400.0;

    pub fn new(value: f64) -> Result<Self, ZoneError> {
        // A NaN fails every comparison, so it has to be refused by name.
        if !value.is_finite() || !(Self::MIN..=Self::MAX).contains(&value) {
            return Err(ZoneError::RainOutOfRange);
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
    fn a_dry_month_and_a_very_wet_month_are_both_allowed() {
        assert!(RainPctOfNormal::new(0.0).is_ok());
        assert!(RainPctOfNormal::new(400.0).is_ok());
        assert_eq!(RainPctOfNormal::new(87.5).expect("rain").value(), 87.5);
    }

    #[test]
    fn a_value_outside_the_range_is_refused() {
        assert!(matches!(
            RainPctOfNormal::new(-0.1),
            Err(ZoneError::RainOutOfRange)
        ));
        assert!(matches!(
            RainPctOfNormal::new(400.1),
            Err(ZoneError::RainOutOfRange)
        ));
    }

    #[test]
    fn not_a_number_is_refused() {
        assert!(RainPctOfNormal::new(f64::NAN).is_err());
        assert!(RainPctOfNormal::new(f64::INFINITY).is_err());
    }
}
