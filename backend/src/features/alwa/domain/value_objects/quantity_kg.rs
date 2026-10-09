use crate::{features::alwa::domain::AlwaError, shared::DomainError};

const MAX_KG: i64 = 1_000_000;

/// How much is on sale or asked for: a whole number of the product's unit.
/// It began as kilograms of a crop and keeps that name; 1 to 1,000 tonnes
/// is the widest range any unit has, and a listing narrows it to its own
/// unit's ([`Unit::max_quantity`](crate::features::alwa::domain::Unit::max_quantity)).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct QuantityKg(i32);

impl QuantityKg {
    pub fn new(value: i64) -> Result<Self, AlwaError> {
        if !(1..=MAX_KG).contains(&value) {
            return Err(DomainError::InvalidValue(format!(
                "Quantity must be a whole number, 1 to {MAX_KG}"
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
