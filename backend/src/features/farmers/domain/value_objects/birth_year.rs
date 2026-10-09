use crate::{features::farmers::domain::FarmerError, shared::DomainError};

const EARLIEST: i32 = 1900;

/// The year a farmer was born, as printed on a support letter.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BirthYear(i32);

impl BirthYear {
    /// `current_year` is passed in so the rule can be tested: nobody is
    /// born in a year that has not begun.
    pub fn new(value: i32, current_year: i32) -> Result<Self, FarmerError> {
        if !(EARLIEST..=current_year).contains(&value) {
            return Err(DomainError::InvalidValue(format!(
                "Birth year must be from {EARLIEST} to {current_year}"
            ))
            .into());
        }

        Ok(Self(value))
    }

    /// Reconstruct from persisted state. A year stored last year is still
    /// valid this year, so only the lower bound could ever be broken.
    pub fn rehydrate(value: i32) -> Self {
        Self(value)
    }

    pub fn value(&self) -> i32 {
        self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_year_from_1900_to_the_current_one_is_accepted() {
        assert!(BirthYear::new(1900, 2026).is_ok());
        assert!(BirthYear::new(1975, 2026).is_ok());
        assert!(BirthYear::new(2026, 2026).is_ok());
    }

    #[test]
    fn a_year_before_1900_or_after_the_current_one_is_refused() {
        assert!(BirthYear::new(1899, 2026).is_err());
        assert!(BirthYear::new(2027, 2026).is_err());
        assert!(BirthYear::new(-5, 2026).is_err());
    }
}
