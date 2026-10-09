use sea_orm::ActiveValue::{NotSet, Set};

use crate::features::rules::{
    app::AppError,
    domain::{ChangeReason, Rule, RuleChange, RuleCode, RuleUser},
    infra::persistence::postgres::entities::{rule_changes, rules},
};

impl TryFrom<rules::Model> for Rule {
    type Error = AppError;

    fn try_from(model: rules::Model) -> Result<Self, Self::Error> {
        Ok(Rule::rehydrate(
            RuleCode::new(model.code)?,
            model.grp,
            model.name_en,
            model.name_ku,
            model.meaning_en,
            model.meaning_ku,
            model.value,
            model.unit,
            model.min_value,
            model.max_value,
            model.default_value,
            RuleUser::try_from(model.used_by.as_str())?,
            model.updated_by,
            model.updated_at.and_utc(),
        ))
    }
}

impl TryFrom<rule_changes::Model> for RuleChange {
    type Error = AppError;

    fn try_from(model: rule_changes::Model) -> Result<Self, Self::Error> {
        Ok(RuleChange::rehydrate(
            model.id,
            RuleCode::new(model.code)?,
            model.old_value,
            model.new_value,
            ChangeReason::new(model.reason)?,
            model.staff_id,
            model.at.and_utc(),
        ))
    }
}

impl From<&RuleChange> for rule_changes::ActiveModel {
    fn from(change: &RuleChange) -> Self {
        rule_changes::ActiveModel {
            id: match *change.id() {
                Some(id) => Set(id),
                None => NotSet,
            },
            code: Set(change.code().into()),
            old_value: Set(*change.old_value()),
            new_value: Set(*change.new_value()),
            reason: Set(change.reason().into()),
            staff_id: Set(*change.staff_id()),
            at: Set(change.at().naive_utc()),
        }
    }
}
