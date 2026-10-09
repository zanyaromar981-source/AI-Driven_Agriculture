use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use validator::Validate;

use crate::features::plans::{
    app::{AppError, use_cases::RecordFarmPlanInput},
    domain::{
        self, DailyValues, FarmPlan, PlanAlert, PlanCoverage, PlanDecision, PlanSource, PlanText,
    },
};

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum PlanAlertType {
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

impl From<PlanAlertType> for domain::AlertType {
    fn from(value: PlanAlertType) -> Self {
        match value {
            PlanAlertType::Frost => domain::AlertType::Frost,
            PlanAlertType::Heat => domain::AlertType::Heat,
            PlanAlertType::HeavyRain => domain::AlertType::HeavyRain,
            PlanAlertType::DrySpell => domain::AlertType::DrySpell,
            PlanAlertType::RustWeather => domain::AlertType::RustWeather,
            PlanAlertType::SunnPest => domain::AlertType::SunnPest,
            PlanAlertType::Dust => domain::AlertType::Dust,
            PlanAlertType::SprayWindow => domain::AlertType::SprayWindow,
            PlanAlertType::SowingRain => domain::AlertType::SowingRain,
            PlanAlertType::UreaRain => domain::AlertType::UreaRain,
        }
    }
}

impl From<domain::AlertType> for PlanAlertType {
    fn from(value: domain::AlertType) -> Self {
        match value {
            domain::AlertType::Frost => PlanAlertType::Frost,
            domain::AlertType::Heat => PlanAlertType::Heat,
            domain::AlertType::HeavyRain => PlanAlertType::HeavyRain,
            domain::AlertType::DrySpell => PlanAlertType::DrySpell,
            domain::AlertType::RustWeather => PlanAlertType::RustWeather,
            domain::AlertType::SunnPest => PlanAlertType::SunnPest,
            domain::AlertType::Dust => PlanAlertType::Dust,
            domain::AlertType::SprayWindow => PlanAlertType::SprayWindow,
            domain::AlertType::SowingRain => PlanAlertType::SowingRain,
            domain::AlertType::UreaRain => PlanAlertType::UreaRain,
        }
    }
}

/// `watch` is an amber dot on the day, `alarm` a red one.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum PlanAlertLevel {
    Watch,
    Alarm,
}

impl From<PlanAlertLevel> for domain::AlertLevel {
    fn from(value: PlanAlertLevel) -> Self {
        match value {
            PlanAlertLevel::Watch => domain::AlertLevel::Watch,
            PlanAlertLevel::Alarm => domain::AlertLevel::Alarm,
        }
    }
}

impl From<domain::AlertLevel> for PlanAlertLevel {
    fn from(value: domain::AlertLevel) -> Self {
        match value {
            domain::AlertLevel::Watch => PlanAlertLevel::Watch,
            domain::AlertLevel::Alarm => PlanAlertLevel::Alarm,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum PlanDecisionCode {
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

impl From<PlanDecisionCode> for domain::DecisionCode {
    fn from(value: PlanDecisionCode) -> Self {
        match value {
            PlanDecisionCode::SowWait => domain::DecisionCode::SowWait,
            PlanDecisionCode::SowGo => domain::DecisionCode::SowGo,
            PlanDecisionCode::UreaGo => domain::DecisionCode::UreaGo,
            PlanDecisionCode::UreaHold => domain::DecisionCode::UreaHold,
            PlanDecisionCode::SprayOk => domain::DecisionCode::SprayOk,
            PlanDecisionCode::CheckRust => domain::DecisionCode::CheckRust,
            PlanDecisionCode::CountSunnPest => domain::DecisionCode::CountSunnPest,
            PlanDecisionCode::FrostCheck => domain::DecisionCode::FrostCheck,
            PlanDecisionCode::HeatCheck => domain::DecisionCode::HeatCheck,
            PlanDecisionCode::DustDelay => domain::DecisionCode::DustDelay,
        }
    }
}

impl From<domain::DecisionCode> for PlanDecisionCode {
    fn from(value: domain::DecisionCode) -> Self {
        match value {
            domain::DecisionCode::SowWait => PlanDecisionCode::SowWait,
            domain::DecisionCode::SowGo => PlanDecisionCode::SowGo,
            domain::DecisionCode::UreaGo => PlanDecisionCode::UreaGo,
            domain::DecisionCode::UreaHold => PlanDecisionCode::UreaHold,
            domain::DecisionCode::SprayOk => PlanDecisionCode::SprayOk,
            domain::DecisionCode::CheckRust => PlanDecisionCode::CheckRust,
            domain::DecisionCode::CountSunnPest => PlanDecisionCode::CountSunnPest,
            domain::DecisionCode::FrostCheck => PlanDecisionCode::FrostCheck,
            domain::DecisionCode::HeatCheck => PlanDecisionCode::HeatCheck,
            domain::DecisionCode::DustDelay => PlanDecisionCode::DustDelay,
        }
    }
}

/// A warning on one day of the plan.
#[derive(Serialize, Deserialize, Debug, Clone, ToSchema)]
pub struct PlanAlertParams {
    #[serde(rename = "type")]
    pub alert_type: PlanAlertType,
    /// A day the plan covers.
    pub day: NaiveDate,
    /// The number behind the alert, when it has one.
    pub value: Option<f64>,
    pub level: PlanAlertLevel,
    /// 1 to 300 characters.
    pub ku: String,
    /// 1 to 300 characters.
    pub en: String,
}

impl PlanAlertParams {
    fn into_alert(self) -> Result<PlanAlert, AppError> {
        Ok(PlanAlert::new(
            self.alert_type.into(),
            self.day,
            self.value,
            self.level.into(),
            PlanText::new(self.ku)?,
            PlanText::new(self.en)?,
        )?)
    }
}

/// One thing to do, or not to do. A code appears at most once in a plan.
#[derive(Serialize, Deserialize, Debug, Clone, ToSchema)]
pub struct PlanDecisionParams {
    pub code: PlanDecisionCode,
    /// 1 to 300 characters.
    pub ku: String,
    /// 1 to 300 characters.
    pub en: String,
}

impl PlanDecisionParams {
    fn into_decision(self) -> Result<PlanDecision, AppError> {
        Ok(PlanDecision::new(
            self.code.into(),
            PlanText::new(self.ku)?,
            PlanText::new(self.en)?,
        ))
    }
}

/// A farm's plan, in the shape the app reads it. It replaces the plan the
/// farm already has as a whole, unless that one was issued later.
#[derive(Serialize, Deserialize, Validate, Debug, Clone, ToSchema)]
pub struct RecordFarmPlanParams {
    /// The day the first entry of the daily lists is for, in Iraq.
    pub from: NaiveDate,
    /// Optional; when sent it must be the length of the daily lists.
    pub days: Option<usize>,
    /// 1 to 10 entries, one per day; `null` where there is no number.
    pub rain_mm: Vec<Option<f64>>,
    /// As many entries as `rain_mm`.
    pub tmin: Vec<Option<f64>>,
    /// As many entries as `rain_mm`.
    pub tmax: Vec<Option<f64>>,
    #[serde(default)]
    pub alerts: Vec<PlanAlertParams>,
    #[serde(default)]
    pub decisions: Vec<PlanDecisionParams>,
    /// Where the forecast comes from, 1 to 120 characters.
    pub source: String,
    /// When the forecast was fetched. Not in the future.
    pub issued: DateTime<Utc>,
}

impl RecordFarmPlanParams {
    pub fn into_input(self, farm_id: i32) -> Result<RecordFarmPlanInput, AppError> {
        Ok(RecordFarmPlanInput {
            farm_id,
            from: self.from,
            issued: self.issued,
            // First, so a list that is far too long is refused before any
            // of its entries is looked at.
            daily: DailyValues::new(self.rain_mm, self.tmin, self.tmax, self.days)?,
            alerts: self
                .alerts
                .into_iter()
                .map(PlanAlertParams::into_alert)
                .collect::<Result<Vec<_>, _>>()?,
            decisions: self
                .decisions
                .into_iter()
                .map(PlanDecisionParams::into_decision)
                .collect::<Result<Vec<_>, _>>()?,
            source: PlanSource::new(self.source)?,
        })
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, ToSchema)]
pub struct PlanAlertResponse {
    #[serde(rename = "type")]
    pub alert_type: PlanAlertType,
    pub day: NaiveDate,
    pub value: Option<f64>,
    pub level: PlanAlertLevel,
    pub ku: String,
    pub en: String,
}

impl From<&PlanAlert> for PlanAlertResponse {
    fn from(alert: &PlanAlert) -> Self {
        Self {
            alert_type: (*alert.alert_type()).into(),
            day: *alert.day(),
            value: *alert.value(),
            level: (*alert.level()).into(),
            ku: alert.ku().into(),
            en: alert.en().into(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, ToSchema)]
pub struct PlanDecisionResponse {
    pub code: PlanDecisionCode,
    pub ku: String,
    pub en: String,
}

impl From<&PlanDecision> for PlanDecisionResponse {
    fn from(decision: &PlanDecision) -> Self {
        Self {
            code: (*decision.code()).into(),
            ku: decision.ku().into(),
            en: decision.en().into(),
        }
    }
}

/// The plan exactly as BACKEND.md 2.4 gives it to the app. `days` is the
/// length of the three daily lists, never more than 10.
#[derive(Serialize, Deserialize, Debug, Clone, ToSchema)]
pub struct FarmPlanResponse {
    pub from: NaiveDate,
    pub days: usize,
    pub rain_mm: Vec<Option<f64>>,
    pub tmin: Vec<Option<f64>>,
    pub tmax: Vec<Option<f64>>,
    pub alerts: Vec<PlanAlertResponse>,
    pub decisions: Vec<PlanDecisionResponse>,
    pub source: String,
    pub issued: DateTime<Utc>,
}

impl From<&FarmPlan> for FarmPlanResponse {
    fn from(plan: &FarmPlan) -> Self {
        Self {
            from: *plan.from(),
            days: plan.daily().days(),
            rain_mm: plan.daily().rain_mm().clone(),
            tmin: plan.daily().tmin().clone(),
            tmax: plan.daily().tmax().clone(),
            alerts: plan.alerts().iter().map(Into::into).collect(),
            decisions: plan.decisions().iter().map(Into::into).collect(),
            source: plan.source().into(),
            issued: *plan.issued(),
        }
    }
}

/// One farm as the plan job sees it: where it is and when its plan was
/// issued. `lat` and `lon` are the farm's centroid; `issued` is `null` for
/// a farm with no plan yet. The owner is deliberately not part of it.
#[derive(Serialize, Deserialize, Debug, Clone, ToSchema)]
pub struct PlanCoverageFarmResponse {
    pub id: String,
    pub lat: f64,
    pub lon: f64,
    pub issued: Option<DateTime<Utc>>,
}

impl From<&PlanCoverage> for PlanCoverageFarmResponse {
    fn from(coverage: &PlanCoverage) -> Self {
        Self {
            id: coverage.site().farm_id().to_string(),
            lat: *coverage.site().lat(),
            lon: *coverage.site().lon(),
            issued: *coverage.issued(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, ToSchema)]
pub struct PlanCoverageResponse {
    pub farms: Vec<PlanCoverageFarmResponse>,
}

/// A stored plan as staff see it: every day the job pushed, past ones too.
#[derive(Serialize, Deserialize, Debug, Clone, ToSchema)]
pub struct PlanDashboardStoredResponse {
    #[serde(flatten)]
    pub plan: FarmPlanResponse,
    /// True when the plan is too old for the farmer to be shown it.
    pub stale: bool,
    pub updated_at: DateTime<Utc>,
}

/// `plan` is `null` for a farm that has no plan yet.
#[derive(Serialize, Deserialize, Debug, Clone, ToSchema)]
pub struct PlanDashboardResponse {
    pub farm_id: String,
    pub plan: Option<PlanDashboardStoredResponse>,
}
