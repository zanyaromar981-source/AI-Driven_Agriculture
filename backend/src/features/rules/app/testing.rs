use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use chrono::{DateTime, TimeZone, Utc};

use crate::{
    app::{Pagination, StaffContext},
    features::rules::{
        app::{AppError, RuleRepository},
        domain::{ChangeOutcome, ChangeReason, NewValue, Rule, RuleChange, RuleCode, RuleUser},
    },
};

#[derive(Clone, Debug, PartialEq)]
pub enum RepositoryCall {
    FindAll {
        used_by: Option<RuleUser>,
    },
    FindByCode {
        code: String,
    },
    ChangeValue {
        code: String,
        requested: NewValue,
        reason: String,
        staff_id: i32,
    },
    FindChangesPage {
        code: String,
        page: u64,
        rows_per_page: u64,
    },
}

#[derive(Debug, Default)]
struct Script {
    rules: Vec<Rule>,
    changes: Vec<RuleChange>,
    fail_with_database_error: bool,
}

#[derive(Debug, Clone, Default)]
pub struct FakeRuleRepository {
    script: Arc<Mutex<Script>>,
    calls: Arc<Mutex<Vec<RepositoryCall>>>,
}

impl FakeRuleRepository {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn holding(rules: Vec<Rule>) -> Self {
        let fake = Self::new();
        fake.script.lock().expect("script lock").rules = rules;
        fake
    }

    pub fn failing() -> Self {
        let fake = Self::new();
        fake.script
            .lock()
            .expect("script lock")
            .fail_with_database_error = true;
        fake
    }

    pub fn calls(&self) -> Vec<RepositoryCall> {
        self.calls.lock().expect("calls lock").clone()
    }

    /// The log as stored, oldest first.
    pub fn changes(&self) -> Vec<RuleChange> {
        self.script.lock().expect("script lock").changes.clone()
    }

    pub fn stored(&self, code: &str) -> Option<Rule> {
        self.script
            .lock()
            .expect("script lock")
            .rules
            .iter()
            .find(|rule| rule.code().as_str() == code)
            .cloned()
    }

    fn record(&self, call: RepositoryCall) {
        self.calls.lock().expect("calls lock").push(call);
    }

    fn guard(&self) -> Result<(), AppError> {
        if self
            .script
            .lock()
            .expect("script lock")
            .fail_with_database_error
        {
            return Err(crate::app::AppError::DatabaseError("fake".to_string()).into());
        }

        Ok(())
    }
}

#[async_trait]
impl RuleRepository for FakeRuleRepository {
    async fn find_all(&self, used_by: Option<RuleUser>) -> Result<Vec<Rule>, AppError> {
        self.record(RepositoryCall::FindAll { used_by });
        self.guard()?;

        let script = self.script.lock().expect("script lock");

        Ok(script
            .rules
            .iter()
            .filter(|rule| used_by.is_none_or(|user| *rule.used_by() == user))
            .cloned()
            .collect())
    }

    async fn find_by_code(&self, code: &RuleCode) -> Result<Option<Rule>, AppError> {
        self.record(RepositoryCall::FindByCode {
            code: String::from(code),
        });
        self.guard()?;

        let script = self.script.lock().expect("script lock");

        Ok(script
            .rules
            .iter()
            .find(|rule| rule.code() == code)
            .cloned())
    }

    async fn change_value(
        &self,
        code: &RuleCode,
        requested: NewValue,
        reason: &ChangeReason,
        staff_id: i32,
        now: DateTime<Utc>,
    ) -> Result<Option<ChangeOutcome>, AppError> {
        self.record(RepositoryCall::ChangeValue {
            code: String::from(code),
            requested,
            reason: String::from(reason),
            staff_id,
        });
        self.guard()?;

        // The one mutex stands in for the row lock of the real repository.
        let mut script = self.script.lock().expect("script lock");

        let Some(position) = script.rules.iter().position(|rule| rule.code() == code) else {
            return Ok(None);
        };

        let current = script.rules[position].clone();

        let Some(value) = current.decide(requested)? else {
            return Ok(Some(ChangeOutcome::new(current, None)));
        };

        let id = script.changes.len() as i32 + 1;
        let change = RuleChange::rehydrate(
            id,
            code.clone(),
            *current.value(),
            value,
            reason.clone(),
            staff_id,
            now,
        );
        let changed = current.changed_to(value, staff_id, now);

        script.rules[position] = changed.clone();
        script.changes.push(change.clone());

        Ok(Some(ChangeOutcome::new(changed, Some(change))))
    }

    async fn find_changes_page(
        &self,
        code: &RuleCode,
        pagination: &Pagination,
    ) -> Result<(Vec<RuleChange>, u64), AppError> {
        self.record(RepositoryCall::FindChangesPage {
            code: String::from(code),
            page: *pagination.page(),
            rows_per_page: *pagination.rows_per_page(),
        });
        self.guard()?;

        let script = self.script.lock().expect("script lock");

        let mut matching: Vec<RuleChange> = script
            .changes
            .iter()
            .filter(|change| change.code() == code)
            .cloned()
            .collect();
        matching.reverse();

        let count = matching.len() as u64;

        Ok((
            matching
                .into_iter()
                .skip(pagination.skip() as usize)
                .take(*pagination.rows_per_page() as usize)
                .collect(),
            count,
        ))
    }
}

pub fn staff() -> StaffContext {
    StaffContext::new(
        7,
        "officer@example.org".to_string(),
        std::collections::HashSet::new(),
    )
}

pub fn code(value: &str) -> RuleCode {
    RuleCode::new(value.to_string()).expect("code")
}

pub fn reason(value: &str) -> ChangeReason {
    ChangeReason::new(value.to_string()).expect("reason")
}

pub fn seeded_at() -> DateTime<Utc> {
    Utc.with_ymd_and_hms(2026, 10, 9, 21, 0, 0).unwrap()
}

fn a_rule(code_value: &str, value: f64, min: f64, max: f64, used_by: RuleUser) -> Rule {
    Rule::rehydrate(
        code(code_value),
        "test".to_string(),
        code_value.to_string(),
        None,
        format!("What {code_value} means."),
        None,
        value,
        "c".to_string(),
        min,
        max,
        value,
        used_by,
        None,
        seeded_at(),
    )
}

/// Frost at 0 C, allowed from -1 to 5, read by the weather planner.
pub fn frost() -> Rule {
    a_rule("frost_c", 0.0, -1.0, 5.0, RuleUser::WeatherPlanner)
}

/// Heat at 31 C, allowed from 25 to 45, read by the weather planner.
pub fn heat() -> Rule {
    a_rule("heat_c", 31.0, 25.0, 45.0, RuleUser::WeatherPlanner)
}

/// The edge where "dry" starts, at 60, read by the dryness bands.
pub fn dry_from() -> Rule {
    a_rule("dryness_dry_from", 60.0, 56.0, 75.0, RuleUser::Dryness)
}
