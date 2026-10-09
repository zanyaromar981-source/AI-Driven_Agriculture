use std::sync::Arc;

use crate::{
    app::StaffContext,
    features::outlooks::{
        app::{AppError, OutlookRepository, use_cases::RecordOutlookRunInput},
        domain::OutlookRun,
    },
};

pub struct CreateOutlookRunUseCase {
    repository: Arc<dyn OutlookRepository>,
}

impl CreateOutlookRunUseCase {
    pub fn new(repository: Arc<dyn OutlookRepository>) -> Self {
        Self { repository }
    }

    /// Stores a track record a staff member typed in for a season and issue
    /// that has none. Whether the key is free is not looked up first: the
    /// unique index answers that when the record is stored.
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

        let created =
            self.repository
                .create_run(&run)
                .await?
                .ok_or_else(|| AppError::RunAlreadyExists {
                    season: String::from(run.season()),
                    issued: String::from(run.issued()),
                })?;

        tracing::info!(
            staff_id = *actor.staff_id(),
            season = created.season().as_str(),
            issued = %created.issued().first_day(),
            seasons_tested = *created.seasons_tested(),
            seasons_right = *created.seasons_right(),
            method = created.method().as_str(),
            "outlook run created from the dashboard"
        );

        Ok(created)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::features::outlooks::{
        app::testing::{FakeOutlookRepository, RepositoryCall, a_run, month, season, staff},
        domain::{OutlookError, RunMethod},
    };

    fn input(seasons_right: i32) -> RecordOutlookRunInput {
        RecordOutlookRunInput {
            season: season("2026-27"),
            issued: month("2026-10"),
            seasons_tested: 25,
            seasons_right,
            method: RunMethod::new("analog years".to_string()).expect("method"),
        }
    }

    #[tokio::test]
    async fn creates_the_track_record_under_its_season_and_issue() {
        let repository = FakeOutlookRepository::new();
        let use_case = CreateOutlookRunUseCase::new(Arc::new(repository.clone()));

        let created = use_case.execute(&staff(), input(19)).await.expect("run");

        assert!(created.id().is_some(), "the stored record comes back");
        assert_eq!(*created.seasons_right(), 19);
        assert_eq!(
            repository.calls(),
            vec![RepositoryCall::CreateRun {
                season: "2026-27".to_string(),
                issued: "2026-10".to_string(),
            }],
            "one create-only write, and no lookup before it"
        );
    }

    #[tokio::test]
    async fn a_key_that_already_has_a_record_is_refused() {
        let repository = FakeOutlookRepository::holding(vec![], vec![a_run("2026-27", "2026-10")]);
        let use_case = CreateOutlookRunUseCase::new(Arc::new(repository.clone()));

        let result = use_case.execute(&staff(), input(3)).await;

        assert!(matches!(
            result,
            Err(AppError::RunAlreadyExists { season, issued })
                if season == "2026-27" && issued == "2026-10"
        ));
        assert!(
            !repository
                .calls()
                .iter()
                .any(|call| matches!(call, RepositoryCall::UpsertRun { .. })),
            "a create must never fall back to replacing"
        );
    }

    #[tokio::test]
    async fn more_right_than_tested_is_refused_as_on_ingest_and_not_written() {
        let repository = FakeOutlookRepository::new();
        let use_case = CreateOutlookRunUseCase::new(Arc::new(repository.clone()));

        let result = use_case.execute(&staff(), input(26)).await;

        assert!(matches!(
            result,
            Err(AppError::Outlook(OutlookError::RightOutOfRange { .. }))
        ));
        assert!(repository.calls().is_empty());
    }

    #[tokio::test]
    async fn a_repository_failure_surfaces() {
        let use_case = CreateOutlookRunUseCase::new(Arc::new(FakeOutlookRepository::failing()));

        assert!(use_case.execute(&staff(), input(19)).await.is_err());
    }
}
