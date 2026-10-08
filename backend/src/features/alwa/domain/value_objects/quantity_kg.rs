use crate::{features::alwa::domain::AlwaError, shared::DomainError};

const MAX_KG: i64 = 1_000_000;

/// A weight of crop in whole kilograms, from 1 kg to 1,000 tonnes.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct QuantityKg(i32);

impl QuantityKg {
    pub fn new(value: i64) -> Result<Self, AlwaError> {
        if !(1..=MAX_KG).contains(&value) {
            return Err(
                DomainError::InvalidValue(format!("Quantity must be 1 to {MAX_KG} kg")).into(),
            );
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
    fn accepts_one_kilogram_up_to_a_thousand_tonnes() {
        assert_eq!(QuantityKg::new(1).expect("quantity").value(), 1);
        assert_eq!(
            QuantityKg::new(MAX_KG).expect("quantity").value(),
            1_000_000
        );
    }

    #[test]
    fn rejects_nothing_a_negative_weight_and_too_much() {
        assert!(QuantityKg::new(0).is_err());
        assert!(QuantityKg::new(-5).is_err());
        assert!(QuantityKg::new(MAX_KG + 1).is_err());
        assert!(QuantityKg::new(i64::MAX).is_err());
    }
}
