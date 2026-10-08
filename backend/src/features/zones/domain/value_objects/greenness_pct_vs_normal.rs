use crate::features::zones::domain::ZoneError;

/// How green the zone is against the long-run normal for the same month, in
/// percent: 0 is normal, -100 is bare ground, positive is greener than usual.
#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
pub struct GreennessPctVsNormal(f64);

impl GreennessPctVsNormal {
    pub const MIN: f64 = -100.0;
    pub const MAX: f64 = 300.0;

    pub fn new(value: f64) -> Result<Self, ZoneError> {
        // A NaN fails every comparison, so it has to be refused by name.
        if !value.is_finite() || !(Self::MIN..=Self::MAX).contains(&value) {
            return Err(ZoneError::GreennessOutOfRange);
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
    fn below_and_above_normal_are_both_allowed() {
        assert!(GreennessPctVsNormal::new(-100.0).is_ok());
        assert!(GreennessPctVsNormal::new(300.0).is_ok());
        assert_eq!(
            GreennessPctVsNormal::new(-12.5).expect("greenness").value(),
            -12.5
        );
    }

    #[test]
    fn a_value_outside_the_range_is_refused() {
        assert!(matches!(
            GreennessPctVsNormal::new(-100.1),
            Err(ZoneError::GreennessOutOfRange)
        ));
        assert!(matches!(
            GreennessPctVsNormal::new(300.1),
            Err(ZoneError::GreennessOutOfRange)
        ));
    }

    #[test]
    fn not_a_number_is_refused() {
        assert!(GreennessPctVsNormal::new(f64::NAN).is_err());
    }
}
