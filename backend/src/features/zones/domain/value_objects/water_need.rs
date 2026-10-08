use crate::features::zones::domain::ZoneError;

/// How much irrigation a zone needs this month: 0 to 100, higher is more.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct WaterNeed(i32);

impl WaterNeed {
    pub const MIN: i32 = 0;
    pub const MAX: i32 = 100;

    pub fn new(value: i32) -> Result<Self, ZoneError> {
        if !(Self::MIN..=Self::MAX).contains(&value) {
            return Err(ZoneError::WaterNeedOutOfRange);
        }

        Ok(Self(value))
    }

    /// A whole JSON number such as `40.0` is taken; a fraction is refused.
    pub fn from_number(value: f64) -> Result<Self, ZoneError> {
        if !value.is_finite() || value.fract() != 0.0 {
            return Err(ZoneError::WaterNeedOutOfRange);
        }

        if value < f64::from(Self::MIN) || value > f64::from(Self::MAX) {
            return Err(ZoneError::WaterNeedOutOfRange);
        }

        Self::new(value as i32)
    }

    pub fn value(&self) -> i32 {
        self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn both_ends_of_the_scale_are_allowed() {
        assert!(WaterNeed::new(0).is_ok());
        assert!(WaterNeed::new(100).is_ok());
    }

    #[test]
    fn a_value_off_the_scale_is_refused() {
        assert!(matches!(
            WaterNeed::new(-1),
            Err(ZoneError::WaterNeedOutOfRange)
        ));
        assert!(matches!(
            WaterNeed::new(101),
            Err(ZoneError::WaterNeedOutOfRange)
        ));
    }

    #[test]
    fn a_whole_number_is_taken_and_a_fraction_refused() {
        assert_eq!(WaterNeed::from_number(40.0).expect("need").value(), 40);
        assert!(WaterNeed::from_number(40.5).is_err());
        assert!(WaterNeed::from_number(f64::INFINITY).is_err());
    }
}
