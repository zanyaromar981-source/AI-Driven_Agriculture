use super::errors::WorkerError;

/// What the cost on a card pays for.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum CostPer {
    #[default]
    Day,
    Hour,
}

impl CostPer {
    pub const ALL: [CostPer; 2] = [CostPer::Day, CostPer::Hour];
}

impl From<CostPer> for String {
    fn from(value: CostPer) -> Self {
        match value {
            CostPer::Day => "day".to_string(),
            CostPer::Hour => "hour".to_string(),
        }
    }
}

impl TryFrom<&str> for CostPer {
    type Error = WorkerError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "day" => Ok(CostPer::Day),
            "hour" => Ok(CostPer::Hour),
            _ => Err(WorkerError::UnknownCostPer),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_cost_per_survives_a_round_trip_through_its_stored_text() {
        for cost_per in CostPer::ALL {
            let stored = String::from(cost_per);

            assert_eq!(CostPer::try_from(stored.as_str()).expect("known"), cost_per);
        }
    }

    #[test]
    fn a_cost_is_per_day_unless_said_otherwise() {
        assert_eq!(CostPer::default(), CostPer::Day);
    }

    #[test]
    fn a_week_is_not_something_a_cost_is_per() {
        assert!(CostPer::try_from("week").is_err());
        assert!(CostPer::try_from("Day").is_err());
    }
}
