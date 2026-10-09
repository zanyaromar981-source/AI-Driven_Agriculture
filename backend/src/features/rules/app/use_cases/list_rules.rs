use std::sync::Arc;

use crate::features::rules::{
    app::{AppError, RuleRepository},
    domain::{Rule, RuleUser},
};

pub struct ListRulesUseCase {
    repository: Arc<dyn RuleRepository>,
}

impl ListRulesUseCase {
    pub fn new(repository: Arc<dyn RuleRepository>) -> Self {
        Self { repository }
    }

    /// Returns the rules as stored: all of them for the dashboard's editing
    /// screen, or only the ones a job reads when it asks for its own.
    pub async fn execute(&self, used_by: Option<RuleUser>) -> Result<Vec<Rule>, AppError> {
        let rules = self.repository.find_all(used_by).await?;

        tracing::debug!(
            used_by = used_by.map(String::from),
            returned = rules.len(),
            "rules listed"
        );

        Ok(rules)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::features::rules::app::testing::{
        FakeRuleRepository, RepositoryCall, dry_from, frost, heat,
    };

    fn repository() -> FakeRuleRepository {
        FakeRuleRepository::holding(vec![frost(), heat(), dry_from()])
    }

    #[tokio::test]
    async fn without_a_reader_every_rule_is_returned() {
        let repository = repository();
        let use_case = ListRulesUseCase::new(Arc::new(repository.clone()));

        let rules = use_case.execute(None).await.expect("rules");

        assert_eq!(rules.len(), 3);
        assert_eq!(
            repository.calls(),
            vec![RepositoryCall::FindAll { used_by: None }]
        );
    }

    #[tokio::test]
    async fn a_job_gets_only_the_rules_it_reads() {
        let repository = repository();
        let use_case = ListRulesUseCase::new(Arc::new(repository.clone()));

        let rules = use_case
            .execute(Some(RuleUser::Dryness))
            .await
            .expect("rules");

        assert_eq!(rules.len(), 1);
        assert_eq!(rules[0].code().as_str(), "dryness_dry_from");
        assert_eq!(
            repository.calls(),
            vec![RepositoryCall::FindAll {
                used_by: Some(RuleUser::Dryness)
            }]
        );
    }

    #[tokio::test]
    async fn a_reader_with_no_rules_gets_an_empty_list() {
        let use_case = ListRulesUseCase::new(Arc::new(repository()));

        let rules = use_case
            .execute(Some(RuleUser::FieldEye))
            .await
            .expect("rules");

        assert!(rules.is_empty());
    }

    #[tokio::test]
    async fn a_repository_failure_surfaces() {
        let use_case = ListRulesUseCase::new(Arc::new(FakeRuleRepository::failing()));

        assert!(use_case.execute(None).await.is_err());
    }
}
