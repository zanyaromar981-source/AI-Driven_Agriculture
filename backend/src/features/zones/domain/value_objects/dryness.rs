use crate::features::zones::domain::{DrynessBand, ZoneError};

/// The dryness index of a zone or sub-zone for one month: 0 to 100, higher
/// is drier.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Dryness(i32);

impl Dryness {
    pub const MIN: i32 = 0;
    pub const MAX: i32 = 100;

    pub fn new(value: i32) -> Result<Self, ZoneError> {
        if !(Self::MIN..=Self::MAX).contains(&value) {
            return Err(ZoneError::DrynessOutOfRange);
        }

        Ok(Self(value))
    }

    /// The data jobs send JSON numbers, which may arrive as `62.0`. A whole
    /// number is taken as it is; a fraction is refused instead of rounded,
    /// because rounding could move a zone across a band edge unseen.
    pub fn from_number(value: f64) -> Result<Self, ZoneError> {
        if !value.is_finite() || value.fract() != 0.0 {
            return Err(ZoneError::DrynessOutOfRange);
        }

        if value < f64::from(Self::MIN) || value > f64::from(Self::MAX) {
            return Err(ZoneError::DrynessOutOfRange);
        }

        Self::new(value as i32)
    }

    pub fn value(&self) -> i32 {
        self.0
    }

    pub fn band(&self) -> DrynessBand {
        DrynessBand::of(self.0)
    }

    /// This dryness minus an earlier one: positive means drier than then.
    pub fn change_from(&self, earlier: Dryness) -> i32 {
        self.0 - earlier.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn both_ends_of_the_scale_are_allowed() {
        assert!(Dryness::new(0).is_ok());
        assert!(Dryness::new(100).is_ok());
    }

    #[test]
    fn a_value_off_the_scale_is_refused() {
        assert!(matches!(
            Dryness::new(-1),
            Err(ZoneError::DrynessOutOfRange)
        ));
        assert!(matches!(
            Dryness::new(101),
            Err(ZoneError::DrynessOutOfRange)
        ));
    }

    #[test]
    fn a_whole_json_number_is_accepted_whatever_way_it_is_written() {
        assert_eq!(Dryness::from_number(62.0).expect("dryness").value(), 62);
    }

    #[test]
    fn a_fraction_is_refused_rather_than_rounded_across_a_band_edge() {
        assert!(Dryness::from_number(59.5).is_err());
        assert!(Dryness::from_number(f64::NAN).is_err());
        assert!(Dryness::from_number(1e12).is_err());
        assert!(Dryness::from_number(-0.5).is_err());
    }

    #[test]
    fn the_band_is_read_off_the_index() {
        assert_eq!(
            Dryness::new(59).expect("dryness").band(),
            DrynessBand::Normal
        );
        assert_eq!(Dryness::new(60).expect("dryness").band(), DrynessBand::Dry);
    }

    #[test]
    fn a_change_is_positive_when_this_year_is_drier() {
        let now = Dryness::new(70).expect("dryness");
        let then = Dryness::new(55).expect("dryness");

        assert_eq!(now.change_from(then), 15);
        assert_eq!(then.change_from(now), -15);
    }
}
