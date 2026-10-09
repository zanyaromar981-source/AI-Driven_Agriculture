use std::sync::Arc;

use crate::{
    app::StaffContext,
    features::outlooks::{
        app::{AppError, OutlookRepository, use_cases::RecordOutlookRunInput},
        domain::OutlookRun,
    },
};

pub struct UpdateOutlookRunUseCase {
    repository: Arc<dyn OutlookRepository>,
}

impl UpdateOutlookRunUseCase {
    pub fn new(repository: Arc<dyn OutlookRepository>) -> Self {
        Self { repository }
    }

    /// Replaces the stored track record of one season and issue. Whether
    /// there is one is not looked up first: the update says how many rows it
    /// touched, and none means not found.
    pub async fn execute(
        &self,
        actor: &StaffContext,
        input: RecordOutlookRunInput,
    ) -> Result<OutlookRun, AppError> {
        let run = OutlookRun::new(
            input.season,
            input.issued,
            input.seasons_tested,
            input.seasons_right,
            input.method,
        )?;

        let updated =
            self.repository
                .update_run(&run)
                .await?
                .ok_or_else(|| AppError::RunNotFound {
                    season: String::from(run.season()),
                    issued: String::from(run.issued()),
                })?;

        tracing::info!(
            staff_id = *actor.staff_id(),
            season = updated.season().as_str(),
            issued = %updated.issued().first_day(),
            seasons_tested = *updated.seasons_tested(),
            seasons_right = *updated.seasons_right(),
            method = updated.method().as_str(),
            "outlook run updated from the dashboard"
        );

        Ok(updated)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::features::outlooks::{
        app::testing::{FakeOutlookRepository, RepositoryCall, a_run, month, season, staff},
        domain::{OutlookError, RunMethod},
    };

    fn input(issued: &str, seasons_right: i32) -> RecordOutlookRunInput {
        RecordOutlookRunInput {
            season: season("2026-27"),
            issued: month(issued),
            seasons_tested: 30,
            seasons_right,
            method: RunMethod::new("analog years, revised".to_string()).expect("method"),
        }
    }

    fn repository() -> FakeOutlookRepository {
        FakeOutlookRepository::holding(vec![], vec![a_run("2026-27", "2026-10")])
    }

    #[tokio::test]
    async fn replaces_the_track_record_of_that_season_and_issue() {
        let repository = repository();
        let use_case = UpdateOutlookRunUseCase::new(Arc::new(repository.clone()));

        let updated = use_case
            .execute(&staff(), input("2026-10", 21))
            .await
            .expect("run");

        assert_eq!(*updated.seasons_tested(), 30);
        assert_eq!(*updated.seasons_right(), 21);
        assert_eq!(updated.method().as_str(), "analog years, revised");
        assert_eq!(
            repository.calls(),
            vec![RepositoryCall::UpdateRun {
                season: "2026-27".to_string(),
                issued: "2026-10".to_string(),
            }],
            "one update-only write, and no lookup before it"
        );
    }

    #[tokio::test]
    async fn a_key_without_a_record_is_not_found_and_none_is_created() {
        let repository = repository();
        let use_case = UpdateOutlookRunUseCase::new(Arc::new(repository.clone()));

        let result = use_case.execute(&staff(), input("2026-09", 21)).await;

        assert!(matches!(
            result,
            Err(AppError::RunNotFound { issued, .. }) if issued == "2026-09"
        ));
        assert!(
            !repository.calls().iter().any(|call| matches!(
                call,
                RepositoryCall::UpsertRun { .. } | RepositoryCall::CreateRun { .. }
            )),
            "an update must never fall back to creating"
        );
    }

    #[tokio::test]
    async fn more_right_than_tested_is_refused_and_not_written() {
        let repository = repository();
        let use_case = UpdateOutlookRunUseCase::new(Arc::new(repository.clone()));

        let result = use_case.execute(&staff(), input("2026-10", 31)).await;

        assert!(matches!(
            result,
            Err(AppError::Outlook(OutlookError::RightOutOfRange { .. }))
        ));
        assert!(repository.calls().is_empty());
    }

    #[tokio::test]
    async fn a_repository_failure_surfaces() {
        let use_case = UpdateOutlookRunUseCase::new(Arc::new(FakeOutlookRepository::failing()));

        assert!(
            use_case
                .execute(&staff(), input("2026-10", 21))
                .await
                .is_err()
        );
    }
}
