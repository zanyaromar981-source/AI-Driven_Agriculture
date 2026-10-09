use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};
use validator::Validate;

use crate::features::alerts::{
    app::{
        AppError,
        use_cases::{RecordAlertInput, RegisterDeviceInput},
    },
    domain::{
        self, Alert, AlertKey, AlertSource, AlertText, Device, DeviceLanguage, HistoryDays,
        PendingPush, PushToken,
    },
};

/// What an alert is about.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, ToSchema)]
#[serde(rename_all = "snake_case")]
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

impl From<AlertType> for domain::AlertType {
    fn from(value: AlertType) -> Self {
        match value {
            AlertType::Frost => domain::AlertType::Frost,
            AlertType::Heat => domain::AlertType::Heat,
            AlertType::HeavyRain => domain::AlertType::HeavyRain,
            AlertType::DrySpell => domain::AlertType::DrySpell,
            AlertType::RustWeather => domain::AlertType::RustWeather,
            AlertType::SunnPest => domain::AlertType::SunnPest,
            AlertType::Dust => domain::AlertType::Dust,
            AlertType::SprayWindow => domain::AlertType::SprayWindow,
            AlertType::SowingRain => domain::AlertType::SowingRain,
            AlertType::UreaRain => domain::AlertType::UreaRain,
            AlertType::Fire => domain::AlertType::Fire,
            AlertType::Brief => domain::AlertType::Brief,
        }
    }
}

impl From<domain::AlertType> for AlertType {
    fn from(value: domain::AlertType) -> Self {
        match value {
            domain::AlertType::Frost => AlertType::Frost,
            domain::AlertType::Heat => AlertType::Heat,
            domain::AlertType::HeavyRain => AlertType::HeavyRain,
            domain::AlertType::DrySpell => AlertType::DrySpell,
            domain::AlertType::RustWeather => AlertType::RustWeather,
            domain::AlertType::SunnPest => AlertType::SunnPest,
            domain::AlertType::Dust => AlertType::Dust,
            domain::AlertType::SprayWindow => AlertType::SprayWindow,
            domain::AlertType::SowingRain => AlertType::SowingRain,
            domain::AlertType::UreaRain => AlertType::UreaRain,
            domain::AlertType::Fire => AlertType::Fire,
            domain::AlertType::Brief => AlertType::Brief,
        }
    }
}

/// `alarm` is red and may be pushed; `watch` is amber and shown in the app only.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum AlertLevel {
    Watch,
    Alarm,
}

impl From<AlertLevel> for domain::AlertLevel {
    fn from(value: AlertLevel) -> Self {
        match value {
            AlertLevel::Watch => domain::AlertLevel::Watch,
            AlertLevel::Alarm => domain::AlertLevel::Alarm,
        }
    }
}

impl From<domain::AlertLevel> for AlertLevel {
    fn from(value: domain::AlertLevel) -> Self {
        match value {
            domain::AlertLevel::Watch => AlertLevel::Watch,
            domain::AlertLevel::Alarm => AlertLevel::Alarm,
        }
    }
}

/// How far the job trusts its own alert.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum AlertConfidence {
    Sure,
    Likely,
    Unsure,
}

impl From<AlertConfidence> for domain::AlertConfidence {
    fn from(value: AlertConfidence) -> Self {
        match value {
            AlertConfidence::Sure => domain::AlertConfidence::Sure,
            AlertConfidence::Likely => domain::AlertConfidence::Likely,
            AlertConfidence::Unsure => domain::AlertConfidence::Unsure,
        }
    }
}

impl From<domain::AlertConfidence> for AlertConfidence {
    fn from(value: domain::AlertConfidence) -> Self {
        match value {
            domain::AlertConfidence::Sure => AlertConfidence::Sure,
            domain::AlertConfidence::Likely => AlertConfidence::Likely,
            domain::AlertConfidence::Unsure => AlertConfidence::Unsure,
        }
    }
}

/// The push service a token belongs to.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum AlertPlatform {
    Android,
    Ios,
}

impl From<AlertPlatform> for domain::Platform {
    fn from(value: AlertPlatform) -> Self {
        match value {
            AlertPlatform::Android => domain::Platform::Android,
            AlertPlatform::Ios => domain::Platform::Ios,
        }
    }
}

impl From<domain::Platform> for AlertPlatform {
    fn from(value: domain::Platform) -> Self {
        match value {
            domain::Platform::Android => AlertPlatform::Android,
            domain::Platform::Ios => AlertPlatform::Ios,
        }
    }
}

#[derive(Deserialize, Debug, Clone, Copy, IntoParams)]
#[into_params(parameter_in = Query)]
pub struct AlertsQueryDto {
    /// How many days back to look, 1 to 90. 30 when left out.
    #[param(example = 30)]
    pub days: Option<i64>,
}

impl AlertsQueryDto {
    pub fn into_days(self) -> Result<HistoryDays, AppError> {
        match self.days {
            Some(days) => Ok(HistoryDays::new(days)?),
            None => Ok(HistoryDays::default()),
        }
    }
}

/// One alert as the farmer app shows it.
#[derive(Serialize, Deserialize, Debug, Clone, ToSchema)]
pub struct AlertResponse {
    pub alert_id: String,
    /// Only in the list across all the farmer's farms.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub farm_id: Option<String>,
    pub r#type: AlertType,
    pub day: NaiveDate,
    pub level: AlertLevel,
    pub confidence: AlertConfidence,
    pub ku: String,
    pub en: String,
    pub action_ku: String,
    pub action_en: String,
    pub pushed: bool,
    pub done: bool,
}

impl AlertResponse {
    fn new(alert: &Alert, with_farm: bool) -> Result<Self, AppError> {
        Ok(Self {
            alert_id: stored_id(alert)?,
            farm_id: with_farm.then(|| alert.farm_id().to_string()),
            r#type: (*alert.alert_type()).into(),
            day: *alert.day(),
            level: (*alert.level()).into(),
            confidence: (*alert.confidence()).into(),
            ku: alert.ku().into(),
            en: alert.en().into(),
            action_ku: alert.action_ku().into(),
            action_en: alert.action_en().into(),
            pushed: *alert.pushed(),
            done: *alert.done(),
        })
    }
}

fn stored_id(alert: &Alert) -> Result<String, AppError> {
    alert
        .id()
        .map(|id| id.to_string())
        .ok_or_else(|| crate::app::AppError::MissingValue("Alert has no id".to_string()).into())
}

#[derive(Serialize, Deserialize, Debug, Clone, ToSchema)]
pub struct AlertsResponse {
    pub alerts: Vec<AlertResponse>,
}

impl AlertsResponse {
    /// The alerts of one farm: the farm is in the path, so it is not
    /// repeated in each alert.
    pub fn of_one_farm(alerts: &[Alert]) -> Result<Self, AppError> {
        Ok(Self {
            alerts: alerts
                .iter()
                .map(|alert| AlertResponse::new(alert, false))
                .collect::<Result<_, _>>()?,
        })
    }

    pub fn of_many_farms(alerts: &[Alert]) -> Result<Self, AppError> {
        Ok(Self {
            alerts: alerts
                .iter()
                .map(|alert| AlertResponse::new(alert, true))
                .collect::<Result<_, _>>()?,
        })
    }
}

/// One alert of a farm as a data job sends it. Sending the same key again
/// replaces all of this and nothing else.
#[derive(Serialize, Deserialize, Validate, Debug, Clone, ToSchema)]
pub struct RecordAlertParams {
    pub r#type: AlertType,
    /// The day the alert is about.
    pub day: NaiveDate,
    pub level: AlertLevel,
    pub confidence: AlertConfidence,
    /// What is happening, in Sorani, 1 to 500 characters.
    pub ku: String,
    /// What is happening, in English, 1 to 500 characters.
    pub en: String,
    /// What to do, in Sorani, 1 to 500 characters.
    pub action_ku: String,
    /// What to do, in English, 1 to 500 characters.
    pub action_en: String,
    /// Which job made the alert and from what, 1 to 120 characters.
    pub source: String,
}

impl RecordAlertParams {
    pub fn into_input(self, farm_id: i32, key: String) -> Result<RecordAlertInput, AppError> {
        Ok(RecordAlertInput {
            farm_id,
            key: AlertKey::new(key)?,
            alert_type: self.r#type.into(),
            day: self.day,
            level: self.level.into(),
            confidence: self.confidence.into(),
            ku: AlertText::new(self.ku)?,
            en: AlertText::new(self.en)?,
            action_ku: AlertText::new(self.action_ku)?,
            action_en: AlertText::new(self.action_en)?,
            source: AlertSource::new(self.source)?,
        })
    }
}

/// A stored alert as staff and the data jobs see it. The owner of the farm
/// is deliberately not part of it.
#[derive(Serialize, Deserialize, Debug, Clone, ToSchema)]
pub struct AlertDashboardResponse {
    pub alert_id: String,
    pub farm_id: String,
    pub key: String,
    pub r#type: AlertType,
    pub day: NaiveDate,
    pub level: AlertLevel,
    pub confidence: AlertConfidence,
    pub ku: String,
    pub en: String,
    pub action_ku: String,
    pub action_en: String,
    pub pushed: bool,
    pub pushed_at: Option<DateTime<Utc>>,
    pub done: bool,
    pub done_at: Option<DateTime<Utc>>,
    pub source: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl TryFrom<&Alert> for AlertDashboardResponse {
    type Error = AppError;

    fn try_from(alert: &Alert) -> Result<Self, Self::Error> {
        Ok(Self {
            alert_id: stored_id(alert)?,
            farm_id: alert.farm_id().to_string(),
            key: alert.key().into(),
            r#type: (*alert.alert_type()).into(),
            day: *alert.day(),
            level: (*alert.level()).into(),
            confidence: (*alert.confidence()).into(),
            ku: alert.ku().into(),
            en: alert.en().into(),
            action_ku: alert.action_ku().into(),
            action_en: alert.action_en().into(),
            pushed: *alert.pushed(),
            pushed_at: *alert.pushed_at(),
            done: *alert.done(),
            done_at: *alert.done_at(),
            source: alert.source().into(),
            created_at: *alert.created_at(),
            updated_at: *alert.updated_at(),
        })
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, ToSchema)]
pub struct AlertDashboardListResponse {
    pub alerts: Vec<AlertDashboardResponse>,
}

impl TryFrom<&[Alert]> for AlertDashboardListResponse {
    type Error = AppError;

    fn try_from(alerts: &[Alert]) -> Result<Self, Self::Error> {
        Ok(Self {
            alerts: alerts
                .iter()
                .map(AlertDashboardResponse::try_from)
                .collect::<Result<_, _>>()?,
        })
    }
}

/// One alarm a push sender may send now: the push body of the contract plus
/// the phones to send it to. No `Debug`: the tokens are secrets.
#[derive(Serialize, Deserialize, Clone, ToSchema)]
pub struct AlertPendingPushResponse {
    pub farm_id: String,
    pub alert_id: String,
    pub r#type: AlertType,
    pub day: NaiveDate,
    pub level: AlertLevel,
    pub confidence: AlertConfidence,
    pub ku: String,
    pub en: String,
    pub action_ku: String,
    pub action_en: String,
    /// The farm owner's phones that want red alerts. May be empty.
    pub devices: Vec<AlertDeviceResponse>,
}

/// One phone to push to. Answered to the push sender only. No `Debug`: the
/// token is a secret.
#[derive(Serialize, Deserialize, Clone, ToSchema)]
pub struct AlertDeviceResponse {
    pub push_token: String,
    pub platform: AlertPlatform,
    /// `ku` or `en`.
    pub lang: String,
    pub notify: AlertNotifyParams,
}

impl From<&Device> for AlertDeviceResponse {
    fn from(device: &Device) -> Self {
        Self {
            push_token: device.push_token().into(),
            platform: (*device.platform()).into(),
            lang: String::from(*device.lang()),
            notify: AlertNotifyParams {
                red_alerts: *device.red_alerts(),
                weekly_plan: *device.weekly_plan(),
            },
        }
    }
}

impl TryFrom<&PendingPush> for AlertPendingPushResponse {
    type Error = AppError;

    fn try_from(pending: &PendingPush) -> Result<Self, Self::Error> {
        let alert = pending.alert();

        Ok(Self {
            farm_id: alert.farm_id().to_string(),
            alert_id: stored_id(alert)?,
            r#type: (*alert.alert_type()).into(),
            day: *alert.day(),
            level: (*alert.level()).into(),
            confidence: (*alert.confidence()).into(),
            ku: alert.ku().into(),
            en: alert.en().into(),
            action_ku: alert.action_ku().into(),
            action_en: alert.action_en().into(),
            devices: pending
                .devices()
                .iter()
                .map(AlertDeviceResponse::from)
                .collect(),
        })
    }
}

#[derive(Serialize, Deserialize, Clone, ToSchema)]
pub struct AlertPendingPushesResponse {
    pub alerts: Vec<AlertPendingPushResponse>,
}

/// Which pushes a phone wants. Both are on when left out.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, ToSchema)]
pub struct AlertNotifyParams {
    #[serde(default = "yes")]
    pub red_alerts: bool,
    #[serde(default = "yes")]
    pub weekly_plan: bool,
}

fn yes() -> bool {
    true
}

/// A phone to push to. No `Debug`: the token is a secret.
#[derive(Serialize, Deserialize, Validate, Clone, ToSchema)]
pub struct RegisterDeviceParams {
    /// 1 to 4,096 characters, as the push service gave it.
    pub push_token: String,
    pub platform: AlertPlatform,
    /// `ku` or `en`. `ku` when left out.
    pub lang: Option<String>,
    pub notify: Option<AlertNotifyParams>,
}

impl RegisterDeviceParams {
    pub fn into_input(self) -> Result<RegisterDeviceInput, AppError> {
        let notify = self.notify.unwrap_or(AlertNotifyParams {
            red_alerts: true,
            weekly_plan: true,
        });

        Ok(RegisterDeviceInput {
            push_token: PushToken::new(self.push_token)?,
            platform: self.platform.into(),
            lang: match self.lang.as_deref() {
                Some(lang) => DeviceLanguage::try_from(lang)?,
                None => DeviceLanguage::Ku,
            },
            red_alerts: notify.red_alerts,
            weekly_plan: notify.weekly_plan,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn device(body: &str) -> RegisterDeviceInput {
        serde_json::from_str::<RegisterDeviceParams>(body)
            .expect("json")
            .into_input()
            .expect("input")
    }

    #[test]
    fn a_device_without_notify_wants_both_kinds_of_push_in_sorani() {
        let input = device(r#"{"push_token": "t", "platform": "android"}"#);

        assert!(input.red_alerts && input.weekly_plan);
        assert_eq!(input.lang, DeviceLanguage::Ku);
    }

    #[test]
    fn a_device_may_switch_one_kind_of_push_off() {
        let input = device(
            r#"{"push_token": "t", "platform": "ios", "lang": "en", "notify": {"red_alerts": false}}"#,
        );

        assert!(!input.red_alerts);
        assert!(input.weekly_plan, "what is left out stays on");
        assert_eq!(input.lang, DeviceLanguage::En);
    }

    #[test]
    fn an_empty_or_oversized_token_and_an_unknown_language_are_refused() {
        let params = |token: String, lang: &str| RegisterDeviceParams {
            push_token: token,
            platform: AlertPlatform::Android,
            lang: Some(lang.to_string()),
            notify: None,
        };

        assert!(params(String::new(), "ku").into_input().is_err());
        assert!(params("a".repeat(4097), "ku").into_input().is_err());
        assert!(params("a".repeat(4096), "ku").into_input().is_ok());
        assert!(params("t".to_string(), "fr").into_input().is_err());
    }

    #[test]
    fn the_days_of_a_list_default_to_thirty_and_stop_at_ninety() {
        assert_eq!(
            AlertsQueryDto { days: None }
                .into_days()
                .expect("days")
                .days(),
            30
        );
        assert!(AlertsQueryDto { days: Some(91) }.into_days().is_err());
        assert!(AlertsQueryDto { days: Some(0) }.into_days().is_err());
    }
}
