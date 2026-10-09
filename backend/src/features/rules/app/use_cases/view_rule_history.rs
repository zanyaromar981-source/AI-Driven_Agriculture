use std::sync::Arc;

use crate::{
    app::Pagination,
    features::rules::{
        app::{AppError, RuleRepository},
        domain::{RuleChange, RuleCode},
    },
};

pub struct ViewRuleHistoryUseCase {
    repository: Arc<dyn RuleRepository>,
}

impl ViewRuleHistoryUseCase {
    pub fn new(repository: Arc<dyn RuleRepository>) -> Self {
        Self { repository }
    }

    /// Returns one page of a rule's changes, newest first, and how many
    /// there are in all. A rule that was never changed has an empty history;
    /// a code that names no rule is not found.
    pub async fn execute(
        &self,
        code: &RuleCode,
        pagination: Pagination,
    ) -> Result<(Vec<RuleChange>, u64), AppError> {
        self.repository
            .find_by_code(code)
            .await?
            .ok_or_else(|| AppError::RuleNotFound(String::from(code)))?;

        let (changes, count) = self.repository.find_changes_page(code, &pagination).await?;

        tracing::debug!(
            rule = code.as_str(),
            returned = changes.len(),
            count,
            "rule history listed"
        );

        Ok((changes, count))
    }
}

#[cfg(test)]
mod tests {
    use chrono::Utc;

    use super::*;
    use crate::features::rules::{
        app::testing::{FakeRuleRepository, RepositoryCall, code, frost, heat, reason},
        domain::NewValue,
    };

    async fn repository_with_three_frost_changes() -> FakeRuleRepository {
        let repository = FakeRuleRepository::holding(vec![frost(), heat()]);

        for (value, staff_id) in [(1.0, 7), (2.0, 8), (3.0, 7)] {
            repository
                .change_value(
                    &code("frost_c"),
                    NewValue::Exactly(value),
                    &reason("trying values"),
                    staff_id,
                    Utc::now(),
                )
                .await
                .expect("change");
        }

        repository
    }

    #[tokio::test]
    async fn the_history_is_newest_first_with_who_made_each_change() {
        let repository = repository_with_three_frost_changes().await;
        let use_case = ViewRuleHistoryUseCase::new(Arc::new(repository.clone()));

        let (changes, count) = use_case
            .execute(&code("frost_c"), Pagination::new(1, 20))
            .await
            .expect("history");

        assert_eq!(count, 3);
        assert_eq!(
            changes
                .iter()
                .map(|change| (*change.new_value(), *change.staff_id()))
                .collect::<Vec<_>>(),
            vec![(3.0, 7), (2.0, 8), (1.0, 7)]
        );
        assert_eq!(
            repository.calls()[3..],
            [
                RepositoryCall::FindByCode {
                    code: "frost_c".to_string()
                },
                RepositoryCall::FindChangesPage {
                    code: "frost_c".to_string(),
                    page: 1,
                    rows_per_page: 20,
                },
            ]
        );
    }

    #[tokio::test]
    async fn a_page_holds_only_its_rows_but_counts_them_all() {
        let repository = repository_with_three_frost_changes().await;
        let use_case = ViewRuleHistoryUseCase::new(Arc::new(repository));

        let (changes, count) = use_case
            .execute(&code("frost_c"), Pagination::new(2, 2))
            .await
            .expect("history");

        assert_eq!(count, 3);
        assert_eq!(changes.len(), 1);
        assert_eq!(
            *changes[0].new_value(),
            1.0,
            "the oldest is on the last page"
        );
    }

    #[tokio::test]
    async fn a_rule_never_changed_has_an_empty_history() {
        let repository = repository_with_three_frost_changes().await;
        let use_case = ViewRuleHistoryUseCase::new(Arc::new(repository));

        let (changes, count) = use_case
            .execute(&code("heat_c"), Pagination::new(1, 20))
            .await
            .expect("history");

        assert!(changes.is_empty(), "another rule's changes must not show");
        assert_eq!(count, 0);
    }

    #[tokio::test]
    async fn an_unknown_rule_is_not_found_rather_than_empty() {
        let repository = FakeRuleRepository::holding(vec![frost()]);
        let use_case = ViewRuleHistoryUseCase::new(Arc::new(repository.clone()));

        let result = use_case
            .execute(&code("frost_f"), Pagination::new(1, 20))
            .await;

        assert!(matches!(result, Err(AppError::RuleNotFound(code)) if code == "frost_f"));
        assert_eq!(
            repository.calls(),
            vec![RepositoryCall::FindByCode {
                code: "frost_f".to_string()
            }]
        );
    }

    #[tokio::test]
    async fn a_repository_failure_surfaces() {
        let use_case = ViewRuleHistoryUseCase::new(Arc::new(FakeRuleRepository::failing()));

        assert!(
            use_case
                .execute(&code("frost_c"), Pagination::new(1, 20))
                .await
                .is_err()
        );
    }
}
