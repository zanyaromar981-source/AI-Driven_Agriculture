use std::sync::Arc;

use crate::{
    app::StaffContext,
    features::outlooks::{
        app::{AppError, OutlookRepository},
        domain::{IssueMonth, Season},
    },
};

pub struct DeleteOutlookRunUseCase {
    repository: Arc<dyn OutlookRepository>,
}

impl DeleteOutlookRunUseCase {
    pub fn new(repository: Arc<dyn OutlookRepository>) -> Self {
        Self { repository }
    }

    /// Removes the track record of one season and issue. Removing one that
    /// is not there succeeds: a repeat of a call whose answer was lost asked
    /// for the record to be gone, and it is.
    pub async fn execute(
        &self,
        actor: &StaffContext,
        season: Season,
        issued: IssueMonth,
    ) -> Result<(), AppError> {
        let removed = self.repository.delete_run(&season, &issued).await?;

        tracing::info!(
            staff_id = *actor.staff_id(),
            season = season.as_str(),
            issued = %issued.first_day(),
            removed,
            "outlook run deleted from the dashboard, or already gone"
        );

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::features::outlooks::app::testing::{
        FakeOutlookRepository, RepositoryCall, a_run, month, season, staff,
    };

    fn repository() -> FakeOutlookRepository {
        FakeOutlookRepository::holding(vec![], vec![a_run("2026-27", "2026-10")])
    }

    #[tokio::test]
    async fn removes_the_record_of_the_named_season_and_issue() {
        let repository = repository();
        let use_case = DeleteOutlookRunUseCase::new(Arc::new(repository.clone()));

        use_case
            .execute(&staff(), season("2026-27"), month("2026-10"))
            .await
            .expect("deleted");

        assert_eq!(
            repository.calls(),
            vec![RepositoryCall::DeleteRun {
                season: "2026-27".to_string(),
                issued: "2026-10".to_string(),
            }]
        );
    }

    #[tokio::test]
    async fn deleting_a_record_that_is_already_gone_succeeds() {
        let use_case = DeleteOutlookRunUseCase::new(Arc::new(repository()));

        let actor = staff();
        let delete = || use_case.execute(&actor, season("2026-27"), month("2026-10"));

        assert!(delete().await.is_ok());
        assert!(delete().await.is_ok(), "a repeated delete must not fail");
    }

    #[tokio::test]
    async fn a_repository_failure_surfaces() {
        let use_case = DeleteOutlookRunUseCase::new(Arc::new(FakeOutlookRepository::failing()));

        assert!(
            use_case
                .execute(&staff(), season("2026-27"), month("2026-10"))
                .await
                .is_err()
        );
    }
}
