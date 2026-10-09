use std::sync::Arc;

use chrono::{DateTime, Utc};

use crate::{
    app::StaffContext,
    features::rules::{
        app::{AppError, RuleRepository},
        domain::{ChangeOutcome, ChangeReason, NewValue, RuleCode},
    },
};

pub struct ChangeRuleInput {
    pub code: RuleCode,
    /// A number the staff member typed, or the rule's default for a reset.
    pub requested: NewValue,
    pub reason: ChangeReason,
}

pub struct ChangeRuleUseCase {
    repository: Arc<dyn RuleRepository>,
}

impl ChangeRuleUseCase {
    pub fn new(repository: Arc<dyn RuleRepository>) -> Self {
        Self { repository }
    }

    /// Sets one rule to a new value, or back to its default, and logs the
    /// change with who made it and why. The rule is not looked up first: the
    /// repository reads it under a lock and the domain decides against that,
    /// so two changes at the same moment log a correct chain. Asking for the
    /// value the rule already holds writes nothing and succeeds.
    pub async fn execute(
        &self,
        actor: &StaffContext,
        input: ChangeRuleInput,
        now: DateTime<Utc>,
    ) -> Result<ChangeOutcome, AppError> {
        let outcome = self
            .repository
            .change_value(
                &input.code,
                input.requested,
                &input.reason,
                *actor.staff_id(),
                now,
            )
            .await?
            .ok_or_else(|| AppError::RuleNotFound(String::from(&input.code)))?;

        match outcome.change() {
            Some(change) => tracing::info!(
                staff_id = *actor.staff_id(),
                rule = input.code.as_str(),
                old_value = *change.old_value(),
                new_value = *change.new_value(),
                reset = input.requested == NewValue::Default,
                "rule changed from the dashboard"
            ),
            None => tracing::info!(
                staff_id = *actor.staff_id(),
                rule = input.code.as_str(),
                value = *outcome.rule().value(),
                "rule already held the requested value: nothing written"
            ),
        }

        Ok(outcome)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::features::rules::{
        app::testing::{
            FakeRuleRepository, RepositoryCall, code, frost, heat, reason, seeded_at, staff,
        },
        domain::RuleError,
    };

    fn input(rule: &str, requested: NewValue, why: &str) -> ChangeRuleInput {
        ChangeRuleInput {
            code: code(rule),
            requested,
            reason: reason(why),
        }
    }

    fn repository() -> FakeRuleRepository {
        FakeRuleRepository::holding(vec![frost(), heat()])
    }

    fn later() -> DateTime<Utc> {
        seeded_at() + chrono::Duration::hours(3)
    }

    #[tokio::test]
    async fn a_change_stores_the_value_and_logs_it_with_who_and_why() {
        let repository = repository();
        let use_case = ChangeRuleUseCase::new(Arc::new(repository.clone()));

        let outcome = use_case
            .execute(
                &staff(),
                input("frost_c", NewValue::Exactly(2.0), "late frost this year"),
                later(),
            )
            .await
            .expect("outcome");

        assert_eq!(*outcome.rule().value(), 2.0);
        assert_eq!(*outcome.rule().updated_by(), Some(7));
        assert_eq!(*outcome.rule().updated_at(), later());

        let log = repository.changes();
        assert_eq!(log.len(), 1);
        assert_eq!(*log[0].old_value(), 0.0);
        assert_eq!(*log[0].new_value(), 2.0);
        assert_eq!(log[0].reason().as_str(), "late frost this year");
        assert_eq!(*log[0].staff_id(), 7);
        assert_eq!(*log[0].at(), later());

        assert_eq!(
            repository.calls(),
            vec![RepositoryCall::ChangeValue {
                code: "frost_c".to_string(),
                requested: NewValue::Exactly(2.0),
                reason: "late frost this year".to_string(),
                staff_id: 7,
            }],
            "the locked write decides: the rule is not looked up before it"
        );
    }

    #[tokio::test]
    async fn changes_one_after_another_log_an_unbroken_chain() {
        let repository = repository();
        let use_case = ChangeRuleUseCase::new(Arc::new(repository.clone()));

        for value in [1.0, 3.0, 2.0] {
            use_case
                .execute(
                    &staff(),
                    input("frost_c", NewValue::Exactly(value), "trying values"),
                    later(),
                )
                .await
                .expect("outcome");
        }

        let chain: Vec<(f64, f64)> = repository
            .changes()
            .iter()
            .map(|change| (*change.old_value(), *change.new_value()))
            .collect();

        assert_eq!(chain, vec![(0.0, 1.0), (1.0, 3.0), (3.0, 2.0)]);
    }

    #[tokio::test]
    async fn the_value_it_already_holds_succeeds_and_writes_nothing() {
        let repository = repository();
        let use_case = ChangeRuleUseCase::new(Arc::new(repository.clone()));

        let outcome = use_case
            .execute(
                &staff(),
                input("frost_c", NewValue::Exactly(0.0), "sent twice"),
                later(),
            )
            .await
            .expect("a repeat is safe");

        assert!(outcome.change().is_none());
        assert_eq!(*outcome.rule().value(), 0.0);
        assert_eq!(
            *outcome.rule().updated_at(),
            seeded_at(),
            "a repeat must not look like a change"
        );
        assert_eq!(*outcome.rule().updated_by(), None);
        assert!(repository.changes().is_empty(), "nothing is logged");
    }

    #[tokio::test]
    async fn a_reset_goes_back_to_the_default_and_is_logged_too() {
        let repository = repository();
        let use_case = ChangeRuleUseCase::new(Arc::new(repository.clone()));

        use_case
            .execute(
                &staff(),
                input("frost_c", NewValue::Exactly(3.0), "trying a value"),
                later(),
            )
            .await
            .expect("outcome");

        let outcome = use_case
            .execute(
                &staff(),
                input("frost_c", NewValue::Default, "back to normal"),
                later(),
            )
            .await
            .expect("outcome");

        assert_eq!(*outcome.rule().value(), 0.0);

        let log = repository.changes();
        assert_eq!(log.len(), 2);
        assert_eq!(*log[1].old_value(), 3.0);
        assert_eq!(*log[1].new_value(), 0.0);
        assert_eq!(log[1].reason().as_str(), "back to normal");
    }

    #[tokio::test]
    async fn a_reset_of_a_rule_at_its_default_writes_nothing() {
        let repository = repository();
        let use_case = ChangeRuleUseCase::new(Arc::new(repository.clone()));

        let outcome = use_case
            .execute(
                &staff(),
                input("heat_c", NewValue::Default, "just checking"),
                later(),
            )
            .await
            .expect("outcome");

        assert!(outcome.change().is_none());
        assert!(repository.changes().is_empty());
    }

    #[tokio::test]
    async fn a_value_outside_the_range_is_refused_and_nothing_is_written() {
        let repository = repository();
        let use_case = ChangeRuleUseCase::new(Arc::new(repository.clone()));

        let result = use_case
            .execute(
                &staff(),
                input("frost_c", NewValue::Exactly(5.5), "too warm"),
                later(),
            )
            .await;

        assert!(matches!(
            result,
            Err(AppError::Rule(RuleError::ValueOutOfRange { .. }))
        ));
        assert!(repository.changes().is_empty());
        assert_eq!(
            *repository.stored("frost_c").expect("rule").value(),
            0.0,
            "the refused value must not be stored"
        );
    }

    #[tokio::test]
    async fn an_unknown_rule_is_not_found_and_nothing_is_written() {
        let repository = repository();
        let use_case = ChangeRuleUseCase::new(Arc::new(repository.clone()));

        let result = use_case
            .execute(
                &staff(),
                input("frost_f", NewValue::Exactly(1.0), "no such rule"),
                later(),
            )
            .await;

        assert!(matches!(result, Err(AppError::RuleNotFound(code)) if code == "frost_f"));
        assert!(repository.changes().is_empty());
    }

    #[tokio::test]
    async fn a_change_of_one_rule_leaves_the_others_alone() {
        let repository = repository();
        let use_case = ChangeRuleUseCase::new(Arc::new(repository.clone()));

        use_case
            .execute(
                &staff(),
                input("frost_c", NewValue::Exactly(2.0), "late frost this year"),
                later(),
            )
            .await
            .expect("outcome");

        assert_eq!(*repository.stored("heat_c").expect("rule").value(), 31.0);
    }

    #[tokio::test]
    async fn a_repository_failure_surfaces() {
        let use_case = ChangeRuleUseCase::new(Arc::new(FakeRuleRepository::failing()));

        assert!(
            use_case
                .execute(
                    &staff(),
                    input("frost_c", NewValue::Exactly(1.0), "any reason"),
                    later(),
                )
                .await
                .is_err()
        );
    }
}
