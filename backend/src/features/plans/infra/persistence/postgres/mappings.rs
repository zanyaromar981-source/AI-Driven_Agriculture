use chrono::{NaiveDate, NaiveDateTime};
use sea_orm::ActiveValue::{NotSet, Set};
use serde::{Deserialize, Serialize};

use crate::{
    app::AppError as GlobalAppError,
    features::plans::{
        app::AppError,
        domain::{
            AlertLevel, AlertType, DailyValues, DecisionCode, FarmPlan, PlanAlert, PlanDecision,
            PlanSource, PlanStamp, PlanText,
        },
        infra::persistence::postgres::entities::farm_plans,
    },
};

/// One alert as it is kept in the `alerts` JSON column.
#[derive(Serialize, Deserialize)]
struct StoredAlert {
    #[serde(rename = "type")]
    alert_type: String,
    day: NaiveDate,
    value: Option<f64>,
    level: String,
    ku: String,
    en: String,
}

/// One decision as it is kept in the `decisions` JSON column.
#[derive(Serialize, Deserialize)]
struct StoredDecision {
    code: String,
    ku: String,
    en: String,
}

fn unreadable(column: &str, error: serde_json::Error) -> AppError {
    GlobalAppError::MissingValue(format!("Stored plan {column} are not readable: {error}")).into()
}

fn stored_alerts(alerts: &[PlanAlert]) -> serde_json::Value {
    let alerts: Vec<StoredAlert> = alerts
        .iter()
        .map(|alert| StoredAlert {
            alert_type: (*alert.alert_type()).into(),
            day: *alert.day(),
            value: *alert.value(),
            level: (*alert.level()).into(),
            ku: alert.ku().into(),
            en: alert.en().into(),
        })
        .collect();

    serde_json::json!(alerts)
}

fn alerts_from(stored: serde_json::Value) -> Result<Vec<PlanAlert>, AppError> {
    let alerts: Vec<StoredAlert> =
        serde_json::from_value(stored).map_err(|error| unreadable("alerts", error))?;

    alerts
        .into_iter()
        .map(|alert| {
            Ok(PlanAlert::new(
                AlertType::try_from(alert.alert_type.as_str())?,
                alert.day,
                alert.value,
                AlertLevel::try_from(alert.level.as_str())?,
                PlanText::new(alert.ku)?,
                PlanText::new(alert.en)?,
            )?)
        })
        .collect()
}

fn stored_decisions(decisions: &[PlanDecision]) -> serde_json::Value {
    let decisions: Vec<StoredDecision> = decisions
        .iter()
        .map(|decision| StoredDecision {
            code: (*decision.code()).into(),
            ku: decision.ku().into(),
            en: decision.en().into(),
        })
        .collect();

    serde_json::json!(decisions)
}

fn decisions_from(stored: serde_json::Value) -> Result<Vec<PlanDecision>, AppError> {
    let decisions: Vec<StoredDecision> =
        serde_json::from_value(stored).map_err(|error| unreadable("decisions", error))?;

    decisions
        .into_iter()
        .map(|decision| {
            Ok(PlanDecision::new(
                DecisionCode::try_from(decision.code.as_str())?,
                PlanText::new(decision.ku)?,
                PlanText::new(decision.en)?,
            ))
        })
        .collect()
}

fn numbers_from(stored: serde_json::Value) -> Result<Vec<Option<f64>>, AppError> {
    serde_json::from_value(stored).map_err(|error| unreadable("daily values", error))
}

impl TryFrom<farm_plans::Model> for FarmPlan {
    type Error = AppError;

    fn try_from(model: farm_plans::Model) -> Result<Self, Self::Error> {
        Ok(FarmPlan::rehydrate(
            model.id,
            model.farm_id,
            model.from_day,
            model.issued.and_utc(),
            DailyValues::new(
                numbers_from(model.rain_mm)?,
                numbers_from(model.tmin)?,
                numbers_from(model.tmax)?,
                None,
            )?,
            alerts_from(model.alerts)?,
            decisions_from(model.decisions)?,
            PlanSource::new(model.source)?,
            model.updated_at.and_utc(),
        ))
    }
}

impl From<&FarmPlan> for farm_plans::ActiveModel {
    fn from(plan: &FarmPlan) -> Self {
        farm_plans::ActiveModel {
            id: match *plan.id() {
                Some(id) => Set(id),
                None => NotSet,
            },
            farm_id: Set(*plan.farm_id()),
            from_day: Set(*plan.from()),
            issued: Set(plan.issued().naive_utc()),
            rain_mm: Set(serde_json::json!(plan.daily().rain_mm())),
            tmin: Set(serde_json::json!(plan.daily().tmin())),
            tmax: Set(serde_json::json!(plan.daily().tmax())),
            alerts: Set(stored_alerts(plan.alerts())),
            decisions: Set(stored_decisions(plan.decisions())),
            source: Set(plan.source().into()),
            updated_at: Set(plan.updated_at().naive_utc()),
        }
    }
}

impl From<(i32, NaiveDateTime)> for PlanStamp {
    fn from((farm_id, issued): (i32, NaiveDateTime)) -> Self {
        PlanStamp::rehydrate(farm_id, issued.and_utc())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn text(value: &str) -> PlanText {
        PlanText::new(value.to_string()).expect("text")
    }

    #[test]
    fn alerts_survive_the_trip_through_the_json_column_in_order() {
        let day = NaiveDate::from_ymd_opt(2026, 10, 11).expect("date");
        let alerts = vec![
            PlanAlert::new(
                AlertType::Frost,
                day,
                Some(-3.2),
                AlertLevel::Alarm,
                text("سەرما"),
                text("Frost -3 °C Sun night"),
            )
            .expect("alert"),
            PlanAlert::new(
                AlertType::RustWeather,
                day,
                None,
                AlertLevel::Watch,
                text("Rust weather"),
                text("Rust weather"),
            )
            .expect("alert"),
        ];

        assert_eq!(alerts_from(stored_alerts(&alerts)).expect("alerts"), alerts);
    }

    #[test]
    fn an_alert_is_stored_under_the_names_the_app_reads() {
        let day = NaiveDate::from_ymd_opt(2026, 10, 11).expect("date");
        let alert = PlanAlert::new(
            AlertType::SowingRain,
            day,
            Some(21.0),
            AlertLevel::Watch,
            text("a"),
            text("b"),
        )
        .expect("alert");

        assert_eq!(
            stored_alerts(&[alert]),
            serde_json::json!([{
                "type": "sowing_rain", "day": "2026-10-11", "value": 21.0,
                "level": "watch", "ku": "a", "en": "b"
            }])
        );
    }

    #[test]
    fn decisions_survive_the_trip_through_the_json_column_in_order() {
        let decisions = vec![
            PlanDecision::new(DecisionCode::SowWait, text("a"), text("b")),
            PlanDecision::new(DecisionCode::SprayOk, text("c"), text("d")),
        ];

        assert_eq!(
            decisions_from(stored_decisions(&decisions)).expect("decisions"),
            decisions
        );
    }

    #[test]
    fn a_day_without_a_number_comes_back_empty() {
        let numbers = numbers_from(serde_json::json!([0.0, null, 2.1])).expect("numbers");

        assert_eq!(numbers, vec![Some(0.0), None, Some(2.1)]);
    }

    #[test]
    fn a_json_column_of_the_wrong_shape_is_an_error_not_an_empty_plan() {
        assert!(alerts_from(serde_json::json!({"type": "frost"})).is_err());
        assert!(alerts_from(serde_json::json!([{"type": "hail"}])).is_err());
        assert!(decisions_from(serde_json::json!([{"code": "sow"}])).is_err());
        assert!(numbers_from(serde_json::json!(["wet"])).is_err());
    }
}
