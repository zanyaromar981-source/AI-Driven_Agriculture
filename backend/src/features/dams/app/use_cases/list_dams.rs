use std::sync::Arc;

use crate::features::dams::{
    app::{AppError, DamRepository},
    domain::{DamStatus, year_ago_window},
};

pub struct ListDamsUseCase {
    repository: Arc<dyn DamRepository>,
}

impl ListDamsUseCase {
    pub fn new(repository: Arc<dyn DamRepository>) -> Self {
        Self { repository }
    }

    /// Returns every dam with its latest reading and the reading from about
    /// a year before it. A dam nobody has measured yet is still listed.
    pub async fn execute(&self) -> Result<Vec<DamStatus>, AppError> {
        let dams = self.repository.find_all().await?;

        let mut statuses = Vec::with_capacity(dams.len());

        // Two short queries per dam. There are only a handful of dams.
        for dam in dams {
            let latest = self.repository.find_latest_reading(*dam.id()).await?;

            let window = latest
                .as_ref()
                .and_then(|latest| year_ago_window(*latest.day()));

            let candidates = match window {
                Some((from, to)) => {
                    self.repository
                        .find_readings_between(*dam.id(), from, to)
                        .await?
                }
                None => Vec::new(),
            };

            statuses.push(DamStatus::new(dam, latest, candidates));
        }

        tracing::debug!(returned = statuses.len(), "dams listed");

        Ok(statuses)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::features::dams::app::testing::{
        FakeDamRepository, RepositoryCall, a_reading, darbandikhan, day, dukan,
    };

    #[tokio::test]
    async fn every_dam_is_listed_even_one_without_readings() {
        let repository = FakeDamRepository::holding(vec![dukan(), darbandikhan()], vec![]);
        let use_case = ListDamsUseCase::new(Arc::new(repository.clone()));

        let statuses = use_case.execute().await.expect("statuses");

        assert_eq!(statuses.len(), 2);
        assert!(statuses.iter().all(|status| status.latest().is_none()));
        assert!(statuses.iter().all(|status| status.year_ago().is_none()));
        assert_eq!(
            repository.calls(),
            vec![
                RepositoryCall::FindAll,
                RepositoryCall::FindLatestReading { dam_id: 1 },
                RepositoryCall::FindLatestReading { dam_id: 2 },
            ],
            "without a latest reading there is no year-ago window to look in"
        );
    }

    #[tokio::test]
    async fn the_latest_reading_comes_with_the_one_from_a_year_before() {
        let repository = FakeDamRepository::holding(
            vec![dukan()],
            vec![
                a_reading(&dukan(), day(2025, 10, 5), 61.0),
                a_reading(&dukan(), day(2026, 9, 1), 40.0),
                a_reading(&dukan(), day(2026, 10, 1), 38.5),
            ],
        );
        let use_case = ListDamsUseCase::new(Arc::new(repository.clone()));

        let statuses = use_case.execute().await.expect("statuses");

        let latest = statuses[0].latest().as_ref().expect("latest");
        let year_ago = statuses[0].year_ago().as_ref().expect("year ago");

        assert_eq!(*latest.day(), day(2026, 10, 1));
        assert_eq!(latest.pct_full().value(), 38.5);
        assert_eq!(*year_ago.day(), day(2025, 10, 5));
        assert_eq!(year_ago.pct_full().value(), 61.0);
        assert_eq!(
            repository.calls(),
            vec![
                RepositoryCall::FindAll,
                RepositoryCall::FindLatestReading { dam_id: 1 },
                RepositoryCall::FindReadingsBetween {
                    dam_id: 1,
                    from: day(2025, 8, 17),
                    to: day(2025, 11, 15),
                },
            ]
        );
    }

    #[tokio::test]
    async fn a_dam_measured_for_less_than_a_year_has_no_year_ago() {
        let repository = FakeDamRepository::holding(
            vec![dukan()],
            vec![
                a_reading(&dukan(), day(2026, 6, 1), 55.0),
                a_reading(&dukan(), day(2026, 10, 1), 38.5),
            ],
        );
        let use_case = ListDamsUseCase::new(Arc::new(repository));

        let statuses = use_case.execute().await.expect("statuses");

        assert!(statuses[0].latest().is_some());
        assert!(
            statuses[0].year_ago().is_none(),
            "a reading four months old must not be passed off as last year's"
        );
    }

    #[tokio::test]
    async fn one_dams_readings_are_never_shown_under_another() {
        let repository = FakeDamRepository::holding(
            vec![dukan(), darbandikhan()],
            vec![a_reading(&darbandikhan(), day(2026, 10, 1), 20.0)],
        );
        let use_case = ListDamsUseCase::new(Arc::new(repository));

        let statuses = use_case.execute().await.expect("statuses");

        assert!(statuses[0].latest().is_none());
        assert!(statuses[1].latest().is_some());
    }

    #[tokio::test]
    async fn a_repository_failure_surfaces() {
        let use_case = ListDamsUseCase::new(Arc::new(FakeDamRepository::failing()));

        assert!(use_case.execute().await.is_err());
    }
}
