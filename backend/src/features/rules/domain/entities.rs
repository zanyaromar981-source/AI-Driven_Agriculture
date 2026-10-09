use chrono::{DateTime, Utc};
use getset::Getters;

use crate::features::rules::domain::{ChangeReason, RuleCode, RuleError, RuleUser};

/// What a staff member asks a rule to become.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum NewValue {
    /// The number they typed.
    Exactly(f64),
    /// The number the rule was seeded with.
    Default,
}

/// One number a data job uses to decide a warning or a colour. Rules exist
/// only by migration, because each one is read by code: staff change the
/// value, never the set of rules.
#[derive(Clone, Debug, PartialEq, Getters)]
#[getset(get = "pub")]
pub struct Rule {
    code: RuleCode,
    grp: String,
    name_en: String,
    /// None until a Sorani speaker has written it.
    name_ku: Option<String>,
    meaning_en: String,
    meaning_ku: Option<String>,
    value: f64,
    unit: String,
    min_value: f64,
    max_value: f64,
    default_value: f64,
    used_by: RuleUser,
    /// The staff member who last changed the value. None = never changed.
    updated_by: Option<i32>,
    updated_at: DateTime<Utc>,
}

impl Rule {
    /// Reconstruct from persisted state.
    #[allow(clippy::too_many_arguments)]
    pub fn rehydrate(
        code: RuleCode,
        grp: String,
        name_en: String,
        name_ku: Option<String>,
        meaning_en: String,
        meaning_ku: Option<String>,
        value: f64,
        unit: String,
        min_value: f64,
        max_value: f64,
        default_value: f64,
        used_by: RuleUser,
        updated_by: Option<i32>,
        updated_at: DateTime<Utc>,
    ) -> Self {
        Self {
            code,
            grp,
            name_en,
            name_ku,
            meaning_en,
            meaning_ku,
            value,
            unit,
            min_value,
            max_value,
            default_value,
            used_by,
            updated_by,
            updated_at,
        }
    }

    /// Decides a requested change against the rule as it is stored now.
    /// `Ok(None)` means the rule already holds that value, so there is
    /// nothing to write and nothing to log: a repeat of a change is safe.
    /// `Ok(Some(value))` is the value to store. A value outside the rule's
    /// own range is refused.
    pub fn decide(&self, requested: NewValue) -> Result<Option<f64>, RuleError> {
        let value = match requested {
            NewValue::Exactly(value) => value,
            NewValue::Default => self.default_value,
        };

        if !value.is_finite() {
            return Err(RuleError::ValueNotANumber);
        }

        if value < self.min_value || value > self.max_value {
            return Err(RuleError::ValueOutOfRange {
                code: String::from(&self.code),
                value,
                min: self.min_value,
                max: self.max_value,
            });
        }

        if value == self.value {
            return Ok(None);
        }

        Ok(Some(value))
    }

    /// The rule after `staff_id` set it to `value` at `now`.
    pub fn changed_to(&self, value: f64, staff_id: i32, now: DateTime<Utc>) -> Self {
        Self {
            value,
            updated_by: Some(staff_id),
            updated_at: now,
            ..self.clone()
        }
    }
}

/// One change of one rule's value: what it was, what it became, who did it
/// and why. Written once and never altered.
#[derive(Clone, Debug, PartialEq, Getters)]
#[getset(get = "pub")]
pub struct RuleChange {
    /// None = new (not yet persisted), Some = existing (persisted)
    id: Option<i32>,
    code: RuleCode,
    old_value: f64,
    new_value: f64,
    reason: ChangeReason,
    staff_id: i32,
    at: DateTime<Utc>,
}

impl RuleChange {
    /// The log entry for `rule` moving from the value it holds to
    /// `new_value`. The old value is taken from the rule itself, so a caller
    /// cannot log a chain that does not match what was stored.
    pub fn of(
        rule: &Rule,
        new_value: f64,
        reason: ChangeReason,
        staff_id: i32,
        at: DateTime<Utc>,
    ) -> Self {
        Self {
            id: None,
            code: rule.code.clone(),
            old_value: rule.value,
            new_value,
            reason,
            staff_id,
            at,
        }
    }

    /// Reconstruct from persisted state.
    #[allow(clippy::too_many_arguments)]
    pub fn rehydrate(
        id: i32,
        code: RuleCode,
        old_value: f64,
        new_value: f64,
        reason: ChangeReason,
        staff_id: i32,
        at: DateTime<Utc>,
    ) -> Self {
        Self {
            id: Some(id),
            code,
            old_value,
            new_value,
            reason,
            staff_id,
            at,
        }
    }
}

/// What a change request did.
#[derive(Clone, Debug, PartialEq, Getters)]
#[getset(get = "pub")]
pub struct ChangeOutcome {
    /// The rule as it is stored after the request.
    rule: Rule,
    /// The log entry written, or None when the rule already held the value.
    change: Option<RuleChange>,
}

impl ChangeOutcome {
    pub fn new(rule: Rule, change: Option<RuleChange>) -> Self {
        Self { rule, change }
    }
}

#[cfg(test)]
mod tests {
    use chrono::TimeZone;

    use super::*;

    fn frost() -> Rule {
        Rule::rehydrate(
            RuleCode::new("frost_c".to_string()).expect("code"),
            "temperature".to_string(),
            "Frost night".to_string(),
            None,
            "A night is a frost night at or below this.".to_string(),
            None,
            1.0,
            "c".to_string(),
            -1.0,
            5.0,
            0.0,
            RuleUser::WeatherPlanner,
            None,
            Utc.with_ymd_and_hms(2026, 10, 9, 0, 0, 0).unwrap(),
        )
    }

    fn reason() -> ChangeReason {
        ChangeReason::new("late frost this year".to_string()).expect("reason")
    }

    #[test]
    fn a_value_inside_the_range_is_the_value_to_store() {
        assert_eq!(
            frost().decide(NewValue::Exactly(2.5)).expect("decision"),
            Some(2.5)
        );
    }

    #[test]
    fn both_ends_of_the_range_are_allowed() {
        assert_eq!(
            frost().decide(NewValue::Exactly(-1.0)).expect("decision"),
            Some(-1.0)
        );
        assert_eq!(
            frost().decide(NewValue::Exactly(5.0)).expect("decision"),
            Some(5.0)
        );
    }

    #[test]
    fn a_value_outside_the_range_is_refused_and_the_range_is_named() {
        assert!(matches!(
            frost().decide(NewValue::Exactly(5.1)),
            Err(RuleError::ValueOutOfRange { code, value, min, max })
                if code == "frost_c" && value == 5.1 && min == -1.0 && max == 5.0
        ));
        assert!(frost().decide(NewValue::Exactly(-1.01)).is_err());
    }

    #[test]
    fn something_that_is_not_a_number_is_refused() {
        assert!(matches!(
            frost().decide(NewValue::Exactly(f64::NAN)),
            Err(RuleError::ValueNotANumber)
        ));
        assert!(frost().decide(NewValue::Exactly(f64::INFINITY)).is_err());
    }

    #[test]
    fn the_value_it_already_holds_is_nothing_to_write() {
        assert_eq!(
            frost().decide(NewValue::Exactly(1.0)).expect("decision"),
            None
        );
    }

    #[test]
    fn a_reset_goes_to_the_default() {
        assert_eq!(
            frost().decide(NewValue::Default).expect("decision"),
            Some(0.0)
        );
    }

    #[test]
    fn a_reset_of_a_rule_at_its_default_is_nothing_to_write() {
        let at_default = frost().changed_to(0.0, 7, Utc::now());

        assert_eq!(
            at_default.decide(NewValue::Default).expect("decision"),
            None
        );
    }

    #[test]
    fn a_change_marks_who_and_when_and_touches_nothing_else() {
        let now = Utc.with_ymd_and_hms(2026, 10, 10, 8, 0, 0).unwrap();
        let before = frost();

        let after = before.changed_to(2.0, 7, now);

        assert_eq!(*after.value(), 2.0);
        assert_eq!(*after.updated_by(), Some(7));
        assert_eq!(*after.updated_at(), now);
        assert_eq!(after.default_value(), before.default_value());
        assert_eq!(after.min_value(), before.min_value());
        assert_eq!(after.code(), before.code());
    }

    #[test]
    fn a_log_entry_takes_its_old_value_from_the_rule() {
        let now = Utc.with_ymd_and_hms(2026, 10, 10, 8, 0, 0).unwrap();

        let change = RuleChange::of(&frost(), 2.0, reason(), 7, now);

        assert_eq!(*change.id(), None);
        assert_eq!(change.code().as_str(), "frost_c");
        assert_eq!(*change.old_value(), 1.0);
        assert_eq!(*change.new_value(), 2.0);
        assert_eq!(*change.staff_id(), 7);
        assert_eq!(*change.at(), now);
    }
}
