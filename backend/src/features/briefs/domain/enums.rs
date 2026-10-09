use crate::shared::DomainError;

/// How much attention one point of a brief asks for.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum PointLevel {
    Info,
    Watch,
    Alarm,
}

impl PointLevel {
    pub const ALL: [PointLevel; 3] = [PointLevel::Info, PointLevel::Watch, PointLevel::Alarm];
}

impl From<PointLevel> for String {
    fn from(value: PointLevel) -> Self {
        match value {
            PointLevel::Info => "info".to_string(),
            PointLevel::Watch => "watch".to_string(),
            PointLevel::Alarm => "alarm".to_string(),
        }
    }
}

impl TryFrom<&str> for PointLevel {
    type Error = DomainError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "info" => Ok(PointLevel::Info),
            "watch" => Ok(PointLevel::Watch),
            "alarm" => Ok(PointLevel::Alarm),
            _ => Err(DomainError::InvalidValue(format!(
                "Invalid point level: {value}"
            ))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A mismatch between the two directions corrupts rows silently rather
    /// than failing.
    #[test]
    fn every_level_round_trips_through_its_stored_string() {
        for level in PointLevel::ALL {
            let stored = String::from(level);

            assert_eq!(PointLevel::try_from(stored.as_str()).expect("level"), level);
        }
    }

    #[test]
    fn an_unknown_stored_level_is_rejected_rather_than_defaulted() {
        assert!(PointLevel::try_from("danger").is_err());
        assert!(PointLevel::try_from("Info").is_err());
        assert!(PointLevel::try_from("").is_err());
    }
}
