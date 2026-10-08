use crate::{features::alwa::domain::AlwaError, shared::DomainError};

/// Far above any real price for a kilogram. It keeps a mistyped number out
/// and the value inside the integer column.
const MAX_IQD: i64 = 100_000_000;

/// A price in whole Iraqi dinars for one kilogram.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct PricePerKg(i32);

impl PricePerKg {
    pub fn new(value: i64) -> Result<Self, AlwaError> {
        if !(1..=MAX_IQD).contains(&value) {
            return Err(DomainError::InvalidValue(format!(
                "Price must be 1 to {MAX_IQD} IQD per kg"
            ))
            .into());
        }

        Ok(Self(value as i32))
    }

    pub fn value(&self) -> i32 {
        self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_a_whole_number_of_dinars_above_zero() {
        assert_eq!(PricePerKg::new(750).expect("price").value(), 750);
        assert!(PricePerKg::new(1).is_ok());
    }

    #[test]
    fn rejects_a_free_or_negative_price() {
        assert!(PricePerKg::new(0).is_err());
        assert!(PricePerKg::new(-750).is_err());
    }

    #[test]
    fn rejects_a_number_too_large_to_be_a_price() {
        assert!(PricePerKg::new(MAX_IQD).is_ok());
        assert!(PricePerKg::new(MAX_IQD + 1).is_err());
    }
}
