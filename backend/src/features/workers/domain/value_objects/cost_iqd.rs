use crate::features::workers::domain::WorkerError;

const MIN: i64 = 1_000;
const MAX: i64 = 10_000_000;

/// What a worker asks for a day or an hour, in whole Iraqi dinars. The
/// range keeps out a zero, a typo with too many digits and a price in
/// thousands written without its zeros.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct CostIqd(i32);

impl CostIqd {
    pub fn new(value: i64) -> Result<Self, WorkerError> {
        if !(MIN..=MAX).contains(&value) {
            return Err(WorkerError::BadCost { min: MIN, max: MAX });
        }

        // The range above fits an `i32` many times over.
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
    fn the_range_is_inclusive_at_both_ends() {
        assert_eq!(CostIqd::new(MIN).expect("cost").value(), 1_000);
        assert_eq!(CostIqd::new(MAX).expect("cost").value(), 10_000_000);
    }

    #[test]
    fn a_cost_outside_the_range_is_refused() {
        assert!(CostIqd::new(0).is_err());
        assert!(CostIqd::new(-25_000).is_err());
        assert!(CostIqd::new(MIN - 1).is_err());
        assert!(CostIqd::new(MAX + 1).is_err());
        assert!(CostIqd::new(i64::MAX).is_err());
    }
}
