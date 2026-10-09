use crate::{features::crops::domain::CropError, shared::DomainError};

const MAX: i32 = 100_000;

/// Where a crop stands in lists and pickers: lower comes first.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct SortOrder(i32);

impl SortOrder {
    pub fn new(value: i32) -> Result<Self, CropError> {
        if !(0..=MAX).contains(&value) {
            return Err(DomainError::InvalidValue(format!(
                "The sort order must be between 0 and {MAX}"
            ))
            .into());
        }

        Ok(Self(value))
    }

    pub fn value(&self) -> i32 {
        self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_range_is_inclusive() {
        assert!(SortOrder::new(0).is_ok());
        assert!(SortOrder::new(MAX).is_ok());
        assert!(SortOrder::new(-1).is_err());
        assert!(SortOrder::new(MAX + 1).is_err());
    }
}
