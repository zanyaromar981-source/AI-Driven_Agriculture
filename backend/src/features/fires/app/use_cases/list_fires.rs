use std::sync::Arc;

use chrono::{Duration, Utc};

use crate::features::fires::{
    app::{AppError, FireRepository},
    domain::{Fire, FireSummary, WindowHours},
};

pub struct ListFiresUseCase {
    repository: Arc<dyn FireRepository>,
}

impl ListFiresUseCase {
    pub fn new(repository: Arc<dyn FireRepository>) -> Self {
        Self { repository }
    }

    /// Returns the fires detected inside the window, newest first, and the
    /// totals over exactly those fires.
    pub async fn execute(&self, window: WindowHours) -> Result<(Vec<Fire>, FireSummary), AppError> {
        let since = Utc::now() - Duration::hours(window.hours());

        let fires = self.repository.find_detected_since(since).await?;
        let summary = FireSummary::of(&fires);

        tracing::debug!(
            hours = window.hours(),
            returned = fires.len(),
            active = *summary.active(),
            "fires listed"
        );

        Ok((fires, summary))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::features::fires::{
        app::testing::{FakeFireRepository, RepositoryCall, a_fire},
        domain::FireStatus,
    };

    fn window(hours: i64) -> WindowHours {
        WindowHours::new(hours).expect("window")
    }

    #[tokio::test]
    async fn lists_only_the_fires_detected_inside_the_window_newest_first() {
        let repository = FakeFireRepository::holding(vec![
            a_fire(1, FireStatus::Active, 30),
            a_fire(2, FireStatus::Active, 20),
            a_fire(3, FireStatus::Spreading, 2),
        ]);
        let use_case = ListFiresUseCase::new(Arc::new(repository));

        let (fires, _) = use_case.execute(window(24)).await.expect("listing");

        let ids: Vec<Option<i32>> = fires.iter().map(|fire| *fire.id()).collect();

        assert_eq!(ids, vec![Some(3), Some(2)]);
    }

    #[tokio::test]
    async fn the_repository_is_asked_for_the_window_and_nothing_else() {
        let repository = FakeFireRepository::new();
        let use_case = ListFiresUseCase::new(Arc::new(repository.clone()));

        let before = Utc::now() - Duration::hours(48);
        use_case.execute(window(48)).await.expect("listing");
        let after = Utc::now() - Duration::hours(48);

        let calls = repository.calls();

        assert_eq!(calls.len(), 1);
        assert!(
            matches!(
                calls[0],
                RepositoryCall::FindDetectedSince { since } if since >= before && since <= after
            ),
            "the window must start 48 hours before now, got {calls:?}"
        );
    }

    #[tokio::test]
    async fn a_fire_that_is_out_is_listed_but_not_counted() {
        let repository = FakeFireRepository::holding(vec![
            a_fire(1, FireStatus::Out, 3),
            a_fire(2, FireStatus::UnderControl, 2),
            a_fire(3, FireStatus::Active, 1),
        ]);
        let use_case = ListFiresUseCase::new(Arc::new(repository));

        let (fires, summary) = use_case.execute(window(24)).await.expect("listing");

        assert_eq!(fires.len(), 3, "an out fire inside the window is listed");
        assert_eq!(*summary.active(), 1);
        assert_eq!(*summary.under_control(), 1);
        assert_eq!(*summary.area_ha(), 20.0);
    }

    #[tokio::test]
    async fn the_summary_covers_the_window_not_the_whole_table() {
        let repository = FakeFireRepository::holding(vec![
            a_fire(1, FireStatus::Active, 100),
            a_fire(2, FireStatus::Active, 1),
        ]);
        let use_case = ListFiresUseCase::new(Arc::new(repository));

        let (_, summary) = use_case.execute(window(24)).await.expect("listing");

        assert_eq!(*summary.active(), 1);
    }

    #[tokio::test]
    async fn no_fires_is_an_empty_list_not_an_error() {
        let use_case = ListFiresUseCase::new(Arc::new(FakeFireRepository::new()));

        let (fires, summary) = use_case
            .execute(WindowHours::default())
            .await
            .expect("listing");

        assert!(fires.is_empty());
        assert_eq!(*summary.active(), 0);
    }

    #[tokio::test]
    async fn a_repository_failure_surfaces() {
        let use_case = ListFiresUseCase::new(Arc::new(FakeFireRepository::failing()));

        assert!(use_case.execute(WindowHours::default()).await.is_err());
    }
}
