use std::sync::Arc;

use super::month_or_latest::month_or_latest;
use crate::features::zones::{
    app::{AppError, ZoneRepository},
    domain::{Month, RegionOverview},
};

pub struct ViewRegionOverviewUseCase {
    repository: Arc<dyn ZoneRepository>,
}

impl ViewRegionOverviewUseCase {
    pub fn new(repository: Arc<dyn ZoneRepository>) -> Self {
        Self { repository }
    }

    /// Without a month, shows the latest month that has any zone reading.
    pub async fn execute(&self, month: Option<Month>) -> Result<RegionOverview, AppError> {
        let month = month_or_latest(self.repository.as_ref(), month).await?;

        let zones = self.repository.find_all_zones().await?;

        // One query for both years; the domain tells them apart by month.
        let months: Vec<Month> = [Some(month), month.a_year_earlier()]
            .into_iter()
            .flatten()
            .collect();

        let readings = self.repository.find_readings_in_months(&months).await?;

        let overview = RegionOverview::build(month, zones, &readings, &readings);

        tracing::debug!(
            month = %String::from(month),
            zones = overview.zones().len(),
            zones_with_data = overview.summary().zones_with_data(),
            "region overview built"
        );

        Ok(overview)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::features::zones::app::testing::{
        FakeZoneRepository, RepositoryCall, a_month, a_reading,
    };

    #[tokio::test]
    async fn shows_the_asked_month_with_its_change_from_a_year_earlier() {
        let repository = FakeZoneRepository::seeded().with_readings(vec![
            a_reading(1, "2026-03", 70),
            a_reading(1, "2025-03", 55),
            a_reading(2, "2026-03", 30),
            a_reading(2, "2026-04", 99),
        ]);
        let use_case = ViewRegionOverviewUseCase::new(Arc::new(repository.clone()));

        let overview = use_case
            .execute(Some(a_month("2026-03")))
            .await
            .expect("overview");

        assert_eq!(*overview.month(), a_month("2026-03"));
        assert_eq!(overview.zones().len(), 3, "every zone is listed");
        assert_eq!(*overview.zones()[0].change_vs_last_year(), Some(15));
        assert_eq!(*overview.zones()[0].rank(), Some(1));
        assert_eq!(*overview.zones()[1].rank(), Some(2));
        assert!(overview.zones()[2].reading().is_none());
        assert_eq!(*overview.summary().average_dryness(), Some(50.0));
        assert_eq!(
            repository.calls(),
            vec![
                RepositoryCall::FindAllZones,
                RepositoryCall::FindReadingsInMonths {
                    months: vec!["2026-03".to_string(), "2025-03".to_string()],
                },
            ],
            "an asked month needs no lookup of the latest one"
        );
    }

    #[tokio::test]
    async fn without_a_month_it_shows_the_latest_month_that_has_a_reading() {
        let repository = FakeZoneRepository::seeded().with_readings(vec![
            a_reading(1, "2026-02", 40),
            a_reading(2, "2026-04", 80),
            a_reading(1, "2025-04", 60),
        ]);
        let use_case = ViewRegionOverviewUseCase::new(Arc::new(repository.clone()));

        let overview = use_case.execute(None).await.expect("overview");

        assert_eq!(*overview.month(), a_month("2026-04"));
        assert_eq!(*overview.summary().zones_with_data(), 1);
        assert_eq!(repository.calls()[0], RepositoryCall::FindReadingMonths);
    }

    #[tokio::test]
    async fn before_any_reading_arrives_every_zone_is_listed_with_nothing_measured() {
        let use_case = ViewRegionOverviewUseCase::new(Arc::new(FakeZoneRepository::seeded()));

        let overview = use_case.execute(None).await.expect("overview");

        assert_eq!(overview.zones().len(), 3);
        assert!(overview.zones().iter().all(|row| row.reading().is_none()));
        assert_eq!(*overview.summary().average_dryness(), None);
    }

    #[tokio::test]
    async fn a_repository_failure_surfaces() {
        let use_case = ViewRegionOverviewUseCase::new(Arc::new(FakeZoneRepository::failing()));

        assert!(use_case.execute(Some(a_month("2026-03"))).await.is_err());
    }
}
