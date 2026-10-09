use std::sync::Arc;

use crate::features::outlooks::{
    app::{AppError, OutlookRepository},
    domain::OutlookRun,
};

pub struct ListOutlookRunsUseCase {
    repository: Arc<dyn OutlookRepository>,
}

impl ListOutlookRunsUseCase {
    pub fn new(repository: Arc<dyn OutlookRepository>) -> Self {
        Self { repository }
    }

    /// Returns every stored track record, newest issue first.
    pub async fn execute(&self) -> Result<Vec<OutlookRun>, AppError> {
        let runs = self.repository.find_runs().await?;

        tracing::debug!(returned = runs.len(), "outlook runs listed");

        Ok(runs)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::features::outlooks::app::testing::{FakeOutlookRepository, RepositoryCall, a_run};

    #[tokio::test]
    async fn every_track_record_is_listed_newest_issue_first() {
        let repository = FakeOutlookRepository::holding(
            vec![],
            vec![
                a_run("2026-27", "2026-09"),
                a_run("2025-26", "2025-10"),
                a_run("2026-27", "2026-10"),
            ],
        );
        let use_case = ListOutlookRunsUseCase::new(Arc::new(repository.clone()));

        let runs = use_case.execute().await.expect("runs");

        assert_eq!(
            runs.iter()
                .map(|run| String::from(run.issued()))
                .collect::<Vec<_>>(),
            vec!["2026-10", "2026-09", "2025-10"]
        );
        assert_eq!(repository.calls(), vec![RepositoryCall::FindRuns]);
    }

    #[tokio::test]
    async fn a_repository_failure_surfaces() {
        let use_case = ListOutlookRunsUseCase::new(Arc::new(FakeOutlookRepository::failing()));

        assert!(use_case.execute().await.is_err());
    }
}
