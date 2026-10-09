use crate::{features::rules::domain::RuleError, shared::DomainError};

/// The code that reads a rule, so the dashboard can say where a number
/// applies and each job can ask for its own rules only.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum RuleUser {
    WeatherPlanner,
    Dryness,
    FieldEye,
}

impl RuleUser {
    pub const ALL: [RuleUser; 3] = [
        RuleUser::WeatherPlanner,
        RuleUser::Dryness,
        RuleUser::FieldEye,
    ];
}

impl From<RuleUser> for String {
    fn from(value: RuleUser) -> Self {
        match value {
            RuleUser::WeatherPlanner => "weather_planner".to_string(),
            RuleUser::Dryness => "dryness".to_string(),
            RuleUser::FieldEye => "field_eye".to_string(),
        }
    }
}

impl TryFrom<&str> for RuleUser {
    type Error = RuleError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "weather_planner" => Ok(RuleUser::WeatherPlanner),
            "dryness" => Ok(RuleUser::Dryness),
            "field_eye" => Ok(RuleUser::FieldEye),
            _ => Err(DomainError::InvalidValue(format!(
                "used_by must be weather_planner, dryness or field_eye, not {value}"
            ))
            .into()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_user_round_trips() {
        for user in RuleUser::ALL {
            let stored = String::from(user);
            let parsed = RuleUser::try_from(stored.as_str())
                .unwrap_or_else(|err| panic!("{user:?} stored as {stored:?} but {err:?}"));

            assert_eq!(parsed, user, "{stored:?} did not round trip");
        }
    }

    #[test]
    fn an_unknown_user_is_rejected_rather_than_defaulted() {
        assert!(RuleUser::try_from("doctor").is_err());
        assert!(RuleUser::try_from("Dryness").is_err());
        assert!(RuleUser::try_from("").is_err());
    }
}
