use async_trait::async_trait;
use chrono::{DateTime, Utc};

use crate::{
    app::Pagination,
    features::rules::{
        app::AppError,
        domain::{ChangeOutcome, ChangeReason, NewValue, Rule, RuleChange, RuleCode, RuleUser},
    },
};

/// There is no way to add or remove a rule here, and none to alter or remove
/// a logged change: rules exist by migration, and the log is insert only.
#[async_trait]
pub trait RuleRepository: Send + Sync + std::fmt::Debug {
    /// Every rule, or only those one job reads, grouped by reader and group
    /// and then by code.
    async fn find_all(&self, used_by: Option<RuleUser>) -> Result<Vec<Rule>, AppError>;

    async fn find_by_code(&self, code: &RuleCode) -> Result<Option<Rule>, AppError>;

    /// Changes one rule's value and logs the change, as one unit: the rule
    /// is read under a lock, [`Rule::decide`] decides against that locked
    /// state, and the new value and its log entry are written together or
    /// not at all. Two changes at the same moment therefore run one after
    /// the other, and the second logs the first one's value as its old one.
    ///
    /// Returns None, with nothing written, when there is no such rule. When
    /// the rule already holds the value, nothing is written and the outcome
    /// carries no log entry.
    async fn change_value(
        &self,
        code: &RuleCode,
        requested: NewValue,
        reason: &ChangeReason,
        staff_id: i32,
        now: DateTime<Utc>,
    ) -> Result<Option<ChangeOutcome>, AppError>;

    /// Returns one page of the rule's changes, newest first, and how many
    /// there are in all.
    async fn find_changes_page(
        &self,
        code: &RuleCode,
        pagination: &Pagination,
    ) -> Result<(Vec<RuleChange>, u64), AppError>;
}
