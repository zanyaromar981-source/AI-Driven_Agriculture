use crate::{features::plans::domain::PlanError, shared::DomainError};

/// What an alert warns about. The list is the one the farmer app knows
/// (BACKEND.md 2.4); a new type is added there and here together.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum AlertType {
    Frost,
    Heat,
    HeavyRain,
    DrySpell,
    RustWeather,
    SunnPest,
    Dust,
    SprayWindow,
    SowingRain,
    UreaRain,
}

impl AlertType {
    pub const ALL: [AlertType; 10] = [
        AlertType::Frost,
        AlertType::Heat,
        AlertType::HeavyRain,
        AlertType::DrySpell,
        AlertType::RustWeather,
        AlertType::SunnPest,
        AlertType::Dust,
        AlertType::SprayWindow,
        AlertType::SowingRain,
        AlertType::UreaRain,
    ];
}

impl From<AlertType> for String {
    fn from(value: AlertType) -> Self {
        match value {
            AlertType::Frost => "frost".to_string(),
            AlertType::Heat => "heat".to_string(),
            AlertType::HeavyRain => "heavy_rain".to_string(),
            AlertType::DrySpell => "dry_spell".to_string(),
            AlertType::RustWeather => "rust_weather".to_string(),
            AlertType::SunnPest => "sunn_pest".to_string(),
            AlertType::Dust => "dust".to_string(),
            AlertType::SprayWindow => "spray_window".to_string(),
            AlertType::SowingRain => "sowing_rain".to_string(),
            AlertType::UreaRain => "urea_rain".to_string(),
        }
    }
}

impl TryFrom<&str> for AlertType {
    type Error = PlanError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "frost" => Ok(AlertType::Frost),
            "heat" => Ok(AlertType::Heat),
            "heavy_rain" => Ok(AlertType::HeavyRain),
            "dry_spell" => Ok(AlertType::DrySpell),
            "rust_weather" => Ok(AlertType::RustWeather),
            "sunn_pest" => Ok(AlertType::SunnPest),
            "dust" => Ok(AlertType::Dust),
            "spray_window" => Ok(AlertType::SprayWindow),
            "sowing_rain" => Ok(AlertType::SowingRain),
            "urea_rain" => Ok(AlertType::UreaRain),
            _ => Err(DomainError::InvalidValue(format!("Invalid alert type: {value}")).into()),
        }
    }
}

/// How loud an alert is. The app paints an amber dot for `Watch` and a red
/// one for `Alarm`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum AlertLevel {
    Watch,
    Alarm,
}

impl AlertLevel {
    pub const ALL: [AlertLevel; 2] = [AlertLevel::Watch, AlertLevel::Alarm];
}

impl From<AlertLevel> for String {
    fn from(value: AlertLevel) -> Self {
        match value {
            AlertLevel::Watch => "watch".to_string(),
            AlertLevel::Alarm => "alarm".to_string(),
        }
    }
}

impl TryFrom<&str> for AlertLevel {
    type Error = PlanError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "watch" => Ok(AlertLevel::Watch),
            "alarm" => Ok(AlertLevel::Alarm),
            _ => Err(DomainError::InvalidValue(format!("Invalid alert level: {value}")).into()),
        }
    }
}

/// One thing to do, or not to do, this week. The app picks an icon by the
/// code, so the list is the one in BACKEND.md 2.4.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum DecisionCode {
    SowWait,
    SowGo,
    UreaGo,
    UreaHold,
    SprayOk,
    CheckRust,
    CountSunnPest,
    FrostCheck,
    HeatCheck,
    DustDelay,
}

impl DecisionCode {
    pub const ALL: [DecisionCode; 10] = [
        DecisionCode::SowWait,
        DecisionCode::SowGo,
        DecisionCode::UreaGo,
        DecisionCode::UreaHold,
        DecisionCode::SprayOk,
        DecisionCode::CheckRust,
        DecisionCode::CountSunnPest,
        DecisionCode::FrostCheck,
        DecisionCode::HeatCheck,
        DecisionCode::DustDelay,
    ];
}

impl From<DecisionCode> for String {
    fn from(value: DecisionCode) -> Self {
        match value {
            DecisionCode::SowWait => "sow_wait".to_string(),
            DecisionCode::SowGo => "sow_go".to_string(),
            DecisionCode::UreaGo => "urea_go".to_string(),
            DecisionCode::UreaHold => "urea_hold".to_string(),
            DecisionCode::SprayOk => "spray_ok".to_string(),
            DecisionCode::CheckRust => "check_rust".to_string(),
            DecisionCode::CountSunnPest => "count_sunn_pest".to_string(),
            DecisionCode::FrostCheck => "frost_check".to_string(),
            DecisionCode::HeatCheck => "heat_check".to_string(),
            DecisionCode::DustDelay => "dust_delay".to_string(),
        }
    }
}

impl TryFrom<&str> for DecisionCode {
    type Error = PlanError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "sow_wait" => Ok(DecisionCode::SowWait),
            "sow_go" => Ok(DecisionCode::SowGo),
            "urea_go" => Ok(DecisionCode::UreaGo),
            "urea_hold" => Ok(DecisionCode::UreaHold),
            "spray_ok" => Ok(DecisionCode::SprayOk),
            "check_rust" => Ok(DecisionCode::CheckRust),
            "count_sunn_pest" => Ok(DecisionCode::CountSunnPest),
            "frost_check" => Ok(DecisionCode::FrostCheck),
            "heat_check" => Ok(DecisionCode::HeatCheck),
            "dust_delay" => Ok(DecisionCode::DustDelay),
            _ => Err(DomainError::InvalidValue(format!("Invalid decision code: {value}")).into()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Every variant must survive a trip to the database and back. A mismatch
    /// between the two directions corrupts rows silently rather than failing.
    #[test]
    fn every_alert_type_round_trips() {
        for alert_type in AlertType::ALL {
            let stored = String::from(alert_type);
            let parsed = AlertType::try_from(stored.as_str())
                .unwrap_or_else(|err| panic!("{alert_type:?} stored as {stored:?} but {err:?}"));

            assert_eq!(parsed, alert_type, "{stored:?} did not round trip");
        }
    }

    #[test]
    fn every_alert_level_round_trips() {
        for level in AlertLevel::ALL {
            let stored = String::from(level);
            let parsed = AlertLevel::try_from(stored.as_str())
                .unwrap_or_else(|err| panic!("{level:?} stored as {stored:?} but {err:?}"));

            assert_eq!(parsed, level, "{stored:?} did not round trip");
        }
    }

    #[test]
    fn every_decision_code_round_trips() {
        for code in DecisionCode::ALL {
            let stored = String::from(code);
            let parsed = DecisionCode::try_from(stored.as_str())
                .unwrap_or_else(|err| panic!("{code:?} stored as {stored:?} but {err:?}"));

            assert_eq!(parsed, code, "{stored:?} did not round trip");
        }
    }

    #[test]
    fn the_stored_forms_are_the_ones_the_app_reads() {
        let alert_types: Vec<String> = AlertType::ALL.into_iter().map(String::from).collect();
        let codes: Vec<String> = DecisionCode::ALL.into_iter().map(String::from).collect();

        assert_eq!(
            alert_types,
            vec![
                "frost",
                "heat",
                "heavy_rain",
                "dry_spell",
                "rust_weather",
                "sunn_pest",
                "dust",
                "spray_window",
                "sowing_rain",
                "urea_rain"
            ],
            "the list in BACKEND.md 2.4, letter for letter"
        );
        assert_eq!(
            codes,
            vec![
                "sow_wait",
                "sow_go",
                "urea_go",
                "urea_hold",
                "spray_ok",
                "check_rust",
                "count_sunn_pest",
                "frost_check",
                "heat_check",
                "dust_delay"
            ],
            "the list in BACKEND.md 2.4, letter for letter"
        );
    }

    #[test]
    fn an_unknown_stored_value_is_rejected_rather_than_defaulted() {
        assert!(AlertType::try_from("hail").is_err());
        assert!(AlertType::try_from("Frost").is_err());
        assert!(AlertLevel::try_from("normal").is_err());
        assert!(AlertLevel::try_from("").is_err());
        assert!(DecisionCode::try_from("sow").is_err());
    }
}
