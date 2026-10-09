use std::sync::Arc;

use chrono::NaiveDate;

use crate::features::briefs::{
    app::{AppError, BriefRepository},
    domain::BriefScope,
};

pub struct DeleteBriefUseCase {
    repository: Arc<dyn BriefRepository>,
}

impl DeleteBriefUseCase {
    pub fn new(repository: Arc<dyn BriefRepository>) -> Self {
        Self { repository }
    }

    /// Removes the brief the nightly job pushed for a day and scope. One
    /// that is already gone is a success, so a re-run of the job gets the
    /// same answer as its first run.
    pub async fn execute(&self, day: NaiveDate, scope: &BriefScope) -> Result<(), AppError> {
        self.repository.delete(day, scope).await?;

        tracing::info!(%day, scope = scope.as_str(), "brief deleted");

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::features::briefs::app::testing::{Call, Fakes, a_brief, a_day, a_scope};

    #[tokio::test]
    async fn removes_only_that_day_of_that_scope_in_one_call() {
        let fakes = Fakes::new()
            .with_stored(a_brief(9, "region"))
            .with_stored(a_brief(8, "region"))
            .with_stored(a_brief(9, "kalar"));
        let use_case = DeleteBriefUseCase::new(Arc::new(fakes.clone()));

        use_case
            .execute(a_day(9), &a_scope("region"))
            .await
            .expect("deleted");

        assert!(fakes.stored(a_day(9), "region").is_none());
        assert!(fakes.stored(a_day(8), "region").is_some());
        assert!(fakes.stored(a_day(9), "kalar").is_some());
        assert_eq!(
            fakes.calls(),
            vec![Call::Delete {
                day: a_day(9),
                scope: "region".to_string(),
            }],
            "the brief is not looked up before the delete"
        );
    }

    #[tokio::test]
    async fn deleting_a_brief_that_is_already_gone_succeeds_again() {
        let use_case = DeleteBriefUseCase::new(Arc::new(Fakes::new()));

        use_case
            .execute(a_day(9), &a_scope("region"))
            .await
            .expect("first");
        use_case
            .execute(a_day(9), &a_scope("region"))
            .await
            .expect("repeat");
    }

    #[tokio::test]
    async fn a_repository_failure_surfaces() {
        let use_case = DeleteBriefUseCase::new(Arc::new(Fakes::new().failing()));

        assert!(
            use_case
                .execute(a_day(9), &a_scope("region"))
                .await
                .is_err()
        );
    }
}
