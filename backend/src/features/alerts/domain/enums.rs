use crate::{features::alerts::domain::AlertError, shared::DomainError};

/// What an alert is about. The first ten are the alert types of the weekly
/// plan; `fire` and `brief` come from other jobs.
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
    Fire,
    Brief,
}

impl AlertType {
    pub const ALL: [AlertType; 12] = [
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
        AlertType::Fire,
        AlertType::Brief,
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
            AlertType::Fire => "fire".to_string(),
            AlertType::Brief => "brief".to_string(),
        }
    }
}

impl TryFrom<&str> for AlertType {
    type Error = AlertError;

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
            "fire" => Ok(AlertType::Fire),
            "brief" => Ok(AlertType::Brief),
            _ => Err(DomainError::InvalidValue(format!("Invalid alert type: {value}")).into()),
        }
    }
}

/// How urgent an alert is. Only `alarm` is ever pushed to a phone; `watch`
/// is shown in the app only.
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
    type Error = AlertError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "watch" => Ok(AlertLevel::Watch),
            "alarm" => Ok(AlertLevel::Alarm),
            _ => Err(DomainError::InvalidValue(format!("Invalid alert level: {value}")).into()),
        }
    }
}

/// How far the job trusts its own alert. Every alert says how sure it is, so
/// an unchecked detection is never worded as a fact.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum AlertConfidence {
    Sure,
    Likely,
    Unsure,
}

impl AlertConfidence {
    pub const ALL: [AlertConfidence; 3] = [
        AlertConfidence::Sure,
        AlertConfidence::Likely,
        AlertConfidence::Unsure,
    ];
}

impl From<AlertConfidence> for String {
    fn from(value: AlertConfidence) -> Self {
        match value {
            AlertConfidence::Sure => "sure".to_string(),
            AlertConfidence::Likely => "likely".to_string(),
            AlertConfidence::Unsure => "unsure".to_string(),
        }
    }
}

impl TryFrom<&str> for AlertConfidence {
    type Error = AlertError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "sure" => Ok(AlertConfidence::Sure),
            "likely" => Ok(AlertConfidence::Likely),
            "unsure" => Ok(AlertConfidence::Unsure),
            _ => Err(DomainError::InvalidValue(format!("Invalid confidence: {value}")).into()),
        }
    }
}

/// The push service a token belongs to.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Platform {
    Android,
    Ios,
}

impl Platform {
    pub const ALL: [Platform; 2] = [Platform::Android, Platform::Ios];
}

impl From<Platform> for String {
    fn from(value: Platform) -> Self {
        match value {
            Platform::Android => "android".to_string(),
            Platform::Ios => "ios".to_string(),
        }
    }
}

impl TryFrom<&str> for Platform {
    type Error = AlertError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "android" => Ok(Platform::Android),
            "ios" => Ok(Platform::Ios),
            _ => Err(DomainError::InvalidValue(format!("Invalid platform: {value}")).into()),
        }
    }
}

/// The language a phone wants its pushes in.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum DeviceLanguage {
    Ku,
    En,
}

impl DeviceLanguage {
    pub const ALL: [DeviceLanguage; 2] = [DeviceLanguage::Ku, DeviceLanguage::En];
}

impl From<DeviceLanguage> for String {
    fn from(value: DeviceLanguage) -> Self {
        match value {
            DeviceLanguage::Ku => "ku".to_string(),
            DeviceLanguage::En => "en".to_string(),
        }
    }
}

impl TryFrom<&str> for DeviceLanguage {
    type Error = AlertError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "ku" => Ok(DeviceLanguage::Ku),
            "en" => Ok(DeviceLanguage::En),
            _ => Err(DomainError::InvalidValue(format!("Invalid language: {value}")).into()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_alert_type_survives_a_round_trip_through_its_stored_string() {
        for value in AlertType::ALL {
            let stored = String::from(value);

            assert_eq!(AlertType::try_from(stored.as_str()).expect("known"), value);
        }
    }

    #[test]
    fn an_unknown_alert_type_is_refused() {
        assert!(AlertType::try_from("nonsense").is_err());
        assert!(AlertType::try_from("").is_err());
    }

    #[test]
    fn every_alert_level_survives_a_round_trip_through_its_stored_string() {
        for value in AlertLevel::ALL {
            let stored = String::from(value);

            assert_eq!(AlertLevel::try_from(stored.as_str()).expect("known"), value);
        }
    }

    #[test]
    fn an_unknown_alert_level_is_refused() {
        assert!(AlertLevel::try_from("nonsense").is_err());
        assert!(AlertLevel::try_from("").is_err());
    }

    #[test]
    fn every_alert_confidence_survives_a_round_trip_through_its_stored_string() {
        for value in AlertConfidence::ALL {
            let stored = String::from(value);

            assert_eq!(
                AlertConfidence::try_from(stored.as_str()).expect("known"),
                value
            );
        }
    }

    #[test]
    fn an_unknown_alert_confidence_is_refused() {
        assert!(AlertConfidence::try_from("nonsense").is_err());
        assert!(AlertConfidence::try_from("").is_err());
    }

    #[test]
    fn every_platform_survives_a_round_trip_through_its_stored_string() {
        for value in Platform::ALL {
            let stored = String::from(value);

            assert_eq!(Platform::try_from(stored.as_str()).expect("known"), value);
        }
    }

    #[test]
    fn an_unknown_platform_is_refused() {
        assert!(Platform::try_from("nonsense").is_err());
        assert!(Platform::try_from("").is_err());
    }

    #[test]
    fn every_device_language_survives_a_round_trip_through_its_stored_string() {
        for value in DeviceLanguage::ALL {
            let stored = String::from(value);

            assert_eq!(
                DeviceLanguage::try_from(stored.as_str()).expect("known"),
                value
            );
        }
    }

    #[test]
    fn an_unknown_device_language_is_refused() {
        assert!(DeviceLanguage::try_from("nonsense").is_err());
        assert!(DeviceLanguage::try_from("").is_err());
    }

    #[test]
    fn the_plan_alert_types_and_the_two_extra_ones_are_all_there() {
        assert_eq!(
            AlertType::ALL.len(),
            12,
            "ten plan types plus fire and brief"
        );
        assert_eq!(String::from(AlertType::HeavyRain), "heavy_rain");
    }
}
