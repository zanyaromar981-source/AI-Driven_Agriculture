use std::sync::Arc;

use chrono::{Duration, Utc};

use crate::features::app_config::{
    app::{AppConfigRepository, AppError},
    domain::{VersionUsage, usage_of},
};

/// Farmers not seen for longer than this are not counted.
const WINDOW_DAYS: i64 = 30;

pub struct ListAppVersionsUseCase {
    repository: Arc<dyn AppConfigRepository>,
}

impl ListAppVersionsUseCase {
    pub fn new(repository: Arc<dyn AppConfigRepository>) -> Self {
        Self { repository }
    }

    /// Returns which versions of the app farmers used in the last 30 days,
    /// newest version first. A farmer who updated in that time is counted
    /// once, under the version they used last, so the shares add up to 1.
    pub async fn execute(&self) -> Result<Vec<VersionUsage>, AppError> {
        let since = Utc::now() - Duration::days(WINDOW_DAYS);

        let counts = self.repository.count_farmers_by_version(since).await?;

        Ok(usage_of(counts))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::features::app_config::app::testing::{Call, Fakes, version};

    #[tokio::test]
    async fn lists_the_newest_version_first_with_its_share() {
        let fakes = Fakes::new().with_counts(vec![
            (version("1.9.0"), 1),
            (version("1.10.0"), 2),
            (version("1.0.3"), 1),
        ]);

        let usage = ListAppVersionsUseCase::new(Arc::new(fakes))
            .execute()
            .await
            .expect("usage");

        assert_eq!(usage[0].version().to_string(), "1.10.0");
        assert_eq!(*usage[0].farmers(), 2);
        assert_eq!(*usage[0].share(), 0.5);
        assert_eq!(usage[2].version().to_string(), "1.0.3");
    }

    #[tokio::test]
    async fn only_the_last_30_days_are_asked_for() {
        let fakes = Fakes::new();
        let before = Utc::now() - Duration::days(WINDOW_DAYS);

        ListAppVersionsUseCase::new(Arc::new(fakes.clone()))
            .execute()
            .await
            .expect("usage");

        let after = Utc::now() - Duration::days(WINDOW_DAYS);
        let [Call::CountFarmersByVersion { since }] = fakes.calls()[..] else {
            panic!("expected one count, got {:?}", fakes.calls());
        };

        assert!(before <= since && since <= after);
    }

    #[tokio::test]
    async fn nobody_seen_is_an_empty_list() {
        let usage = ListAppVersionsUseCase::new(Arc::new(Fakes::new()))
            .execute()
            .await
            .expect("usage");

        assert!(usage.is_empty());
    }

    #[tokio::test]
    async fn a_repository_failure_surfaces() {
        let result = ListAppVersionsUseCase::new(Arc::new(Fakes::new().failing()))
            .execute()
            .await;

        assert!(result.is_err());
    }
}
