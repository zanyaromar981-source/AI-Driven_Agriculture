use std::sync::Arc;

use chrono::{DateTime, Utc};

use crate::{
    app::Pagination,
    features::fires::{
        app::{AppError, FireFilter, FireRepository},
        domain::{DetectionSpan, Fire, FireStatus, ZoneSlug},
    },
};

pub struct ListStoredFiresInput {
    pub from: Option<DateTime<Utc>>,
    pub to: Option<DateTime<Utc>>,
    pub status: Option<FireStatus>,
    pub zone_slug: Option<ZoneSlug>,
    pub pagination: Pagination,
}

pub struct ListStoredFiresUseCase {
    repository: Arc<dyn FireRepository>,
}

impl ListStoredFiresUseCase {
    pub fn new(repository: Arc<dyn FireRepository>) -> Self {
        Self { repository }
    }

    /// Returns one page of the stored fires for the dashboard's editing
    /// screen, newest first, and how many match in all.
    pub async fn execute(&self, input: ListStoredFiresInput) -> Result<(Vec<Fire>, u64), AppError> {
        let filter = FireFilter {
            span: DetectionSpan::new(input.from, input.to, Utc::now())?,
            status: input.status,
            zone_slug: input.zone_slug,
        };

        let (fires, count) = self
            .repository
            .find_page(&filter, &input.pagination)
            .await?;

        tracing::debug!(returned = fires.len(), count, "stored fires listed");

        Ok((fires, count))
    }
}

#[cfg(test)]
mod tests {
    use chrono::Duration;

    use super::*;
    use crate::features::fires::app::testing::{FakeFireRepository, RepositoryCall, a_fire};

    fn input() -> ListStoredFiresInput {
        ListStoredFiresInput {
            from: None,
            to: None,
            status: None,
            zone_slug: None,
            pagination: Pagination::new(1, 20),
        }
    }

    fn ids(fires: &[Fire]) -> Vec<Option<i32>> {
        fires.iter().map(|fire| *fire.id()).collect()
    }

    #[tokio::test]
    async fn without_a_span_it_lists_the_last_thirty_days_newest_first() {
        let repository = FakeFireRepository::holding(vec![
            a_fire(1, FireStatus::Active, 24 * 31),
            a_fire(2, FireStatus::Out, 24 * 29),
            a_fire(3, FireStatus::Active, 1),
        ]);
        let use_case = ListStoredFiresUseCase::new(Arc::new(repository));

        let (fires, count) = use_case.execute(input()).await.expect("listing");

        assert_eq!(ids(&fires), vec![Some(3), Some(2)]);
        assert_eq!(count, 2);
    }

    #[tokio::test]
    async fn the_span_the_status_and_the_zone_reach_the_repository_with_the_page() {
        let repository = FakeFireRepository::new();
        let use_case = ListStoredFiresUseCase::new(Arc::new(repository.clone()));
        let from = Utc::now() - Duration::days(90);
        let to = Utc::now() - Duration::days(60);
        let zone = ZoneSlug::new("soran".to_string()).expect("zone");

        use_case
            .execute(ListStoredFiresInput {
                from: Some(from),
                to: Some(to),
                status: Some(FireStatus::Out),
                zone_slug: Some(zone.clone()),
                pagination: Pagination::new(3, 10),
            })
            .await
            .expect("listing");

        assert_eq!(
            repository.calls(),
            vec![RepositoryCall::FindPage {
                filter: FireFilter {
                    span: DetectionSpan::new(Some(from), Some(to), Utc::now()).expect("span"),
                    status: Some(FireStatus::Out),
                    zone_slug: Some(zone),
                },
                page: 3,
                rows_per_page: 10,
            }]
        );
    }

    #[tokio::test]
    async fn the_status_filter_keeps_only_that_status() {
        let repository = FakeFireRepository::holding(vec![
            a_fire(1, FireStatus::Out, 3),
            a_fire(2, FireStatus::Active, 2),
        ]);
        let use_case = ListStoredFiresUseCase::new(Arc::new(repository));

        let (fires, count) = use_case
            .execute(ListStoredFiresInput {
                status: Some(FireStatus::Out),
                ..input()
            })
            .await
            .expect("listing");

        assert_eq!(ids(&fires), vec![Some(1)]);
        assert_eq!(count, 1);
    }

    #[tokio::test]
    async fn the_count_is_of_every_match_not_of_the_page() {
        let repository = FakeFireRepository::holding(vec![
            a_fire(1, FireStatus::Active, 3),
            a_fire(2, FireStatus::Active, 2),
            a_fire(3, FireStatus::Active, 1),
        ]);
        let use_case = ListStoredFiresUseCase::new(Arc::new(repository));

        let (fires, count) = use_case
            .execute(ListStoredFiresInput {
                pagination: Pagination::new(2, 2),
                ..input()
            })
            .await
            .expect("listing");

        assert_eq!(ids(&fires), vec![Some(1)]);
        assert_eq!(count, 3);
    }

    #[tokio::test]
    async fn a_span_that_ends_before_it_starts_is_refused_before_anything_is_read() {
        let repository = FakeFireRepository::new();
        let use_case = ListStoredFiresUseCase::new(Arc::new(repository.clone()));

        let result = use_case
            .execute(ListStoredFiresInput {
                from: Some(Utc::now()),
                to: Some(Utc::now() - Duration::days(1)),
                ..input()
            })
            .await;

        assert!(matches!(result, Err(AppError::Fire(_))));
        assert!(repository.calls().is_empty());
    }

    #[tokio::test]
    async fn a_repository_failure_surfaces() {
        let use_case = ListStoredFiresUseCase::new(Arc::new(FakeFireRepository::failing()));

        assert!(use_case.execute(input()).await.is_err());
    }
}
