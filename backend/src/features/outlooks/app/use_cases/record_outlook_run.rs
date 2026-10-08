use std::sync::Arc;

use crate::features::outlooks::{
    app::{AppError, OutlookRepository},
    domain::{IssueMonth, OutlookRun, RunMethod, Season},
};

pub struct RecordOutlookRunInput {
    pub season: Season,
    pub issued: IssueMonth,
    pub seasons_tested: i32,
    pub seasons_right: i32,
    pub method: RunMethod,
}

pub struct RecordOutlookRunUseCase {
    repository: Arc<dyn OutlookRepository>,
}

impl RecordOutlookRunUseCase {
    pub fn new(repository: Arc<dyn OutlookRepository>) -> Self {
        Self { repository }
    }

    /// Stores the track record of the method behind one issue. Sending the
    /// same season and month again replaces the earlier record.
    pub async fn execute(&self, input: RecordOutlookRunInput) -> Result<OutlookRun, AppError> {
        let run = OutlookRun::new(
            input.season,
            input.issued,
            input.seasons_tested,
            input.seasons_right,
            input.method,
        )?;

        let recorded = self.repository.upsert_run(&run).await?;

        tracing::info!(
            season = recorded.season().as_str(),
            issued = %recorded.issued().first_day(),
            seasons_tested = *recorded.seasons_tested(),
            seasons_right = *recorded.seasons_right(),
            method = recorded.method().as_str(),
            "outlook run recorded"
        );

        Ok(recorded)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::features::outlooks::{
        app::testing::{FakeOutlookRepository, RepositoryCall, month, season},
        domain::OutlookError,
    };

    fn input(seasons_tested: i32, seasons_right: i32) -> RecordOutlookRunInput {
        RecordOutlookRunInput {
            season: season("2026-27"),
            issued: month("2026-10"),
            seasons_tested,
            seasons_right,
            method: RunMethod::new("analog years".to_string()).expect("method"),
        }
    }

    #[tokio::test]
    async fn records_the_track_record_under_its_season_and_month() {
        let repository = FakeOutlookRepository::new();
        let use_case = RecordOutlookRunUseCase::new(Arc::new(repository.clone()));

        let recorded = use_case.execute(input(25, 19)).await.expect("run");

        assert!(recorded.id().is_some(), "the stored run comes back");
        assert_eq!(*recorded.seasons_tested(), 25);
        assert_eq!(*recorded.seasons_right(), 19);
        assert_eq!(recorded.method().as_str(), "analog years");
        assert_eq!(
            repository.calls(),
            vec![RepositoryCall::UpsertRun {
                season: "2026-27".to_string(),
                issued: "2026-10".to_string(),
            }]
        );
    }

    #[tokio::test]
    async fn an_impossible_track_record_is_not_written() {
        let repository = FakeOutlookRepository::new();
        let use_case = RecordOutlookRunUseCase::new(Arc::new(repository.clone()));

        let result = use_case.execute(input(25, 26)).await;

        assert!(matches!(
            result,
            Err(AppError::Outlook(OutlookError::RightOutOfRange {
                tested: 25,
                right: 26
            }))
        ));
        assert!(
            repository.calls().is_empty(),
            "a record that flatters the method must never reach the dashboard"
        );
    }

    #[tokio::test]
    async fn a_repository_failure_surfaces() {
        let use_case = RecordOutlookRunUseCase::new(Arc::new(FakeOutlookRepository::failing()));

        assert!(use_case.execute(input(25, 19)).await.is_err());
    }
}
