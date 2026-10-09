use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};
use validator::Validate;

use crate::features::rules::{
    app::{AppError, use_cases::ChangeRuleInput},
    domain::{self, ChangeOutcome, ChangeReason, NewValue, Rule, RuleChange, RuleCode},
};

/// The code that reads a rule.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum RuleUsedBy {
    WeatherPlanner,
    Dryness,
    FieldEye,
}

impl From<RuleUsedBy> for domain::RuleUser {
    fn from(value: RuleUsedBy) -> Self {
        match value {
            RuleUsedBy::WeatherPlanner => domain::RuleUser::WeatherPlanner,
            RuleUsedBy::Dryness => domain::RuleUser::Dryness,
            RuleUsedBy::FieldEye => domain::RuleUser::FieldEye,
        }
    }
}

impl From<domain::RuleUser> for RuleUsedBy {
    fn from(value: domain::RuleUser) -> Self {
        match value {
            domain::RuleUser::WeatherPlanner => RuleUsedBy::WeatherPlanner,
            domain::RuleUser::Dryness => RuleUsedBy::Dryness,
            domain::RuleUser::FieldEye => RuleUsedBy::FieldEye,
        }
    }
}

/// One rule as stored, for the dashboard's editing screen.
#[derive(Serialize, Deserialize, Debug, Clone, ToSchema)]
pub struct RuleResponse {
    pub code: String,
    /// Rules of one kind share a group, for example `temperature`.
    pub grp: String,
    pub name_en: String,
    /// Null until the Sorani name has been written.
    pub name_ku: Option<String>,
    pub meaning_en: String,
    pub meaning_ku: Option<String>,
    pub value: f64,
    pub unit: String,
    pub min_value: f64,
    pub max_value: f64,
    pub default_value: f64,
    pub used_by: RuleUsedBy,
    /// The staff member who last changed the value. Null = never changed.
    pub updated_by: Option<String>,
    pub updated_at: DateTime<Utc>,
}

impl From<&Rule> for RuleResponse {
    fn from(rule: &Rule) -> Self {
        Self {
            code: rule.code().into(),
            grp: rule.grp().clone(),
            name_en: rule.name_en().clone(),
            name_ku: rule.name_ku().clone(),
            meaning_en: rule.meaning_en().clone(),
            meaning_ku: rule.meaning_ku().clone(),
            value: *rule.value(),
            unit: rule.unit().clone(),
            min_value: *rule.min_value(),
            max_value: *rule.max_value(),
            default_value: *rule.default_value(),
            used_by: (*rule.used_by()).into(),
            updated_by: rule.updated_by().map(|id| id.to_string()),
            updated_at: *rule.updated_at(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, ToSchema)]
pub struct RulesResponse {
    pub rules: Vec<RuleResponse>,
}

/// The rule after a change or a reset.
#[derive(Serialize, Deserialize, Debug, Clone, ToSchema)]
pub struct OneRuleResponse {
    pub rule: RuleResponse,
    /// False when the rule already held the value, so nothing was written
    /// and nothing was added to its history.
    pub changed: bool,
}

impl From<&ChangeOutcome> for OneRuleResponse {
    fn from(outcome: &ChangeOutcome) -> Self {
        Self {
            rule: outcome.rule().into(),
            changed: outcome.change().is_some(),
        }
    }
}

#[derive(Serialize, Deserialize, Validate, Debug, Clone, ToSchema)]
pub struct ChangeRuleParams {
    /// Between the rule's `min_value` and `max_value`, both included.
    pub value: f64,
    /// Why the value is changed: 3 to 500 characters.
    pub reason: String,
}

impl ChangeRuleParams {
    pub fn into_input(self, code: RuleCode) -> Result<ChangeRuleInput, AppError> {
        Ok(ChangeRuleInput {
            code,
            requested: NewValue::Exactly(self.value),
            reason: ChangeReason::new(self.reason)?,
        })
    }
}

#[derive(Serialize, Deserialize, Validate, Debug, Clone, ToSchema)]
pub struct ResetRuleParams {
    /// Why the rule goes back to its default: 3 to 500 characters.
    pub reason: String,
}

impl ResetRuleParams {
    pub fn into_input(self, code: RuleCode) -> Result<ChangeRuleInput, AppError> {
        Ok(ChangeRuleInput {
            code,
            requested: NewValue::Default,
            reason: ChangeReason::new(self.reason)?,
        })
    }
}

/// One logged change of a rule's value.
#[derive(Serialize, Deserialize, Debug, Clone, ToSchema)]
pub struct RuleChangeResponse {
    pub id: String,
    pub code: String,
    pub old_value: f64,
    pub new_value: f64,
    pub reason: String,
    /// The staff member who made the change.
    pub staff_id: String,
    pub at: DateTime<Utc>,
}

impl From<&RuleChange> for RuleChangeResponse {
    fn from(change: &RuleChange) -> Self {
        Self {
            id: change.id().map(|id| id.to_string()).unwrap_or_default(),
            code: change.code().into(),
            old_value: *change.old_value(),
            new_value: *change.new_value(),
            reason: change.reason().into(),
            staff_id: change.staff_id().to_string(),
            at: *change.at(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, ToSchema)]
pub struct RuleHistoryResponse {
    /// Newest first.
    pub changes: Vec<RuleChangeResponse>,
    /// How many changes the rule has in all, on every page.
    pub count: u64,
    pub page: u64,
    pub rows_per_page: u64,
}

/// `?used_by=`. It is read as text so that a reader written wrongly is
/// answered like every other invalid value rather than a bare bad request.
#[derive(Deserialize, Debug, Clone, Default, IntoParams)]
#[into_params(parameter_in = Query)]
pub struct RuleValuesQuery {
    /// `weather_planner`, `dryness` or `field_eye`. Left out = every rule.
    pub used_by: Option<String>,
}

impl RuleValuesQuery {
    pub fn into_input(self) -> Result<Option<domain::RuleUser>, AppError> {
        Ok(self
            .used_by
            .map(|raw| domain::RuleUser::try_from(raw.as_str()))
            .transpose()?)
    }
}

/// What a data job needs of a rule: the number and what it is counted in.
#[derive(Serialize, Deserialize, Debug, Clone, ToSchema)]
pub struct RuleValueResponse {
    pub code: String,
    pub value: f64,
    pub unit: String,
}

impl From<&Rule> for RuleValueResponse {
    fn from(rule: &Rule) -> Self {
        Self {
            code: rule.code().into(),
            value: *rule.value(),
            unit: rule.unit().clone(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, ToSchema)]
pub struct RuleValuesResponse {
    pub rules: Vec<RuleValueResponse>,
}

#[cfg(test)]
mod tests {
    use chrono::TimeZone;

    use super::*;
    use crate::features::rules::domain::RuleError;

    fn code() -> RuleCode {
        RuleCode::new("frost_c".to_string()).expect("code")
    }

    fn frost() -> Rule {
        Rule::rehydrate(
            code(),
            "temperature".to_string(),
            "Frost night".to_string(),
            None,
            "A night is a frost night at or below this.".to_string(),
            None,
            0.0,
            "c".to_string(),
            -1.0,
            5.0,
            0.0,
            domain::RuleUser::WeatherPlanner,
            Some(7),
            Utc.with_ymd_and_hms(2026, 10, 9, 21, 0, 0).unwrap(),
        )
    }

    #[test]
    fn every_reader_survives_the_trip_through_the_dto_and_matches_its_stored_name() {
        for user in domain::RuleUser::ALL {
            assert_eq!(domain::RuleUser::from(RuleUsedBy::from(user)), user);
            assert_eq!(
                serde_json::to_value(RuleUsedBy::from(user)).expect("json"),
                serde_json::json!(String::from(user)),
            );
        }
    }

    #[test]
    fn a_rule_is_sent_with_a_missing_sorani_name_as_null_and_ids_as_text() {
        let json = serde_json::to_value(RuleResponse::from(&frost())).expect("json");

        assert_eq!(json["code"], "frost_c");
        assert_eq!(json["name_ku"], serde_json::Value::Null);
        assert_eq!(json["meaning_ku"], serde_json::Value::Null);
        assert_eq!(json["used_by"], "weather_planner");
        assert_eq!(json["updated_by"], "7");
        assert_eq!(json["updated_at"], "2026-10-09T21:00:00Z");
    }

    #[test]
    fn a_job_is_sent_the_code_the_value_and_the_unit_and_nothing_else() {
        let json = serde_json::to_value(RuleValueResponse::from(&frost())).expect("json");

        assert_eq!(
            json,
            serde_json::json!({"code": "frost_c", "value": 0.0, "unit": "c"})
        );
    }

    #[test]
    fn a_change_carries_the_typed_number_and_a_reset_asks_for_the_default() {
        let change = ChangeRuleParams {
            value: 2.5,
            reason: " late frost this year ".to_string(),
        }
        .into_input(code())
        .expect("input");

        assert_eq!(change.requested, NewValue::Exactly(2.5));
        assert_eq!(change.reason.as_str(), "late frost this year");

        let reset = ResetRuleParams {
            reason: "back to normal".to_string(),
        }
        .into_input(code())
        .expect("input");

        assert_eq!(reset.requested, NewValue::Default);
    }

    #[test]
    fn a_reason_too_short_is_refused_before_anything_is_asked_of_the_database() {
        let result = ChangeRuleParams {
            value: 2.5,
            reason: "no".to_string(),
        }
        .into_input(code());

        assert!(matches!(
            result,
            Err(AppError::Rule(RuleError::ReasonLength { .. }))
        ));
    }

    #[test]
    fn the_reader_filter_is_optional_and_a_wrong_one_is_refused() {
        assert_eq!(
            RuleValuesQuery { used_by: None }
                .into_input()
                .expect("input"),
            None
        );
        assert_eq!(
            RuleValuesQuery {
                used_by: Some("field_eye".to_string())
            }
            .into_input()
            .expect("input"),
            Some(domain::RuleUser::FieldEye)
        );
        assert!(
            RuleValuesQuery {
                used_by: Some("doctor".to_string())
            }
            .into_input()
            .is_err()
        );
    }
}
