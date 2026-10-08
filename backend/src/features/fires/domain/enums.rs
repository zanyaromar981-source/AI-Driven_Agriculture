use crate::{features::fires::domain::FireError, shared::DomainError};

/// Where a fire stands. `Out` fires stay in the table: the dashboard still
/// lists them while they are inside the window it asked for.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum FireStatus {
    Active,
    Spreading,
    UnderControl,
    Out,
}

impl FireStatus {
    pub const ALL: [FireStatus; 4] = [
        FireStatus::Active,
        FireStatus::Spreading,
        FireStatus::UnderControl,
        FireStatus::Out,
    ];

    /// Still burning freely: the fires the dashboard counts as active.
    pub fn is_burning(self) -> bool {
        matches!(self, FireStatus::Active | FireStatus::Spreading)
    }

    pub fn is_out(self) -> bool {
        self == FireStatus::Out
    }
}

impl From<FireStatus> for String {
    fn from(value: FireStatus) -> Self {
        match value {
            FireStatus::Active => "active".to_string(),
            FireStatus::Spreading => "spreading".to_string(),
            FireStatus::UnderControl => "under_control".to_string(),
            FireStatus::Out => "out".to_string(),
        }
    }
}

impl TryFrom<&str> for FireStatus {
    type Error = FireError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "active" => Ok(FireStatus::Active),
            "spreading" => Ok(FireStatus::Spreading),
            "under_control" => Ok(FireStatus::UnderControl),
            "out" => Ok(FireStatus::Out),
            _ => Err(DomainError::InvalidValue(format!("Invalid fire status: {value}")).into()),
        }
    }
}

/// The compass point the wind blows towards, which is the way the fire is
/// pushed. Weather reports usually name where the wind comes from; the job
/// turns that around before it pushes.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum WindDirection {
    N,
    Ne,
    E,
    Se,
    S,
    Sw,
    W,
    Nw,
}

impl WindDirection {
    pub const ALL: [WindDirection; 8] = [
        WindDirection::N,
        WindDirection::Ne,
        WindDirection::E,
        WindDirection::Se,
        WindDirection::S,
        WindDirection::Sw,
        WindDirection::W,
        WindDirection::Nw,
    ];
}

impl From<WindDirection> for String {
    fn from(value: WindDirection) -> Self {
        match value {
            WindDirection::N => "n".to_string(),
            WindDirection::Ne => "ne".to_string(),
            WindDirection::E => "e".to_string(),
            WindDirection::Se => "se".to_string(),
            WindDirection::S => "s".to_string(),
            WindDirection::Sw => "sw".to_string(),
            WindDirection::W => "w".to_string(),
            WindDirection::Nw => "nw".to_string(),
        }
    }
}

impl TryFrom<&str> for WindDirection {
    type Error = FireError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "n" => Ok(WindDirection::N),
            "ne" => Ok(WindDirection::Ne),
            "e" => Ok(WindDirection::E),
            "se" => Ok(WindDirection::Se),
            "s" => Ok(WindDirection::S),
            "sw" => Ok(WindDirection::Sw),
            "w" => Ok(WindDirection::W),
            "nw" => Ok(WindDirection::Nw),
            _ => Err(DomainError::InvalidValue(format!("Invalid wind direction: {value}")).into()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Every variant must survive a trip to the database and back. A mismatch
    /// between the two directions corrupts rows silently rather than failing.
    #[test]
    fn every_status_round_trips() {
        for status in FireStatus::ALL {
            let stored = String::from(status);
            let parsed = FireStatus::try_from(stored.as_str())
                .unwrap_or_else(|err| panic!("{status:?} stored as {stored:?} but {err:?}"));

            assert_eq!(parsed, status, "{stored:?} did not round trip");
        }
    }

    #[test]
    fn every_wind_direction_round_trips() {
        for direction in WindDirection::ALL {
            let stored = String::from(direction);
            let parsed = WindDirection::try_from(stored.as_str())
                .unwrap_or_else(|err| panic!("{direction:?} stored as {stored:?} but {err:?}"));

            assert_eq!(parsed, direction, "{stored:?} did not round trip");
        }
    }

    #[test]
    fn the_stored_form_is_lower_case_snake_case() {
        assert_eq!(String::from(FireStatus::UnderControl), "under_control");
        assert_eq!(String::from(WindDirection::Nw), "nw");
    }

    #[test]
    fn an_unknown_stored_value_is_rejected_rather_than_defaulted() {
        assert!(FireStatus::try_from("burning").is_err());
        assert!(FireStatus::try_from("Active").is_err());
        assert!(FireStatus::try_from("").is_err());
        assert!(WindDirection::try_from("north").is_err());
        assert!(WindDirection::try_from("NE").is_err());
    }

    #[test]
    fn only_active_and_spreading_fires_count_as_burning() {
        assert!(FireStatus::Active.is_burning());
        assert!(FireStatus::Spreading.is_burning());
        assert!(!FireStatus::UnderControl.is_burning());
        assert!(!FireStatus::Out.is_burning());
    }

    #[test]
    fn a_fire_under_control_is_not_out_yet() {
        assert!(!FireStatus::UnderControl.is_out());
        assert!(FireStatus::Out.is_out());
    }
}
