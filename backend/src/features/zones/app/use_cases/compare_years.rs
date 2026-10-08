use std::sync::Arc;

use crate::features::zones::{
    app::{AppError, ZoneRepository},
    domain::{Month, RegionComparison, YearComparison},
};

pub struct CompareYearsUseCase {
    repository: Arc<dyn ZoneRepository>,
}

impl CompareYearsUseCase {
    pub fn new(repository: Arc<dyn ZoneRepository>) -> Self {
        Self { repository }
    }

    pub async fn execute(&self, comparison: YearComparison) -> Result<RegionComparison, AppError> {
        // The region line runs over every year that has data for this
        // calendar month, not only the two being compared.
        let months: Vec<Month> = self
            .repository
            .find_reading_months()
            .await?
            .into_iter()
            .filter(|month| month.number() == comparison.calendar_month())
            .collect();

        let readings = self.repository.find_readings_in_months(&months).await?;
        let zones = self.repository.find_all_zones().await?;

        let compared = RegionComparison::build(comparison, zones, &readings);

        tracing::debug!(
            year = comparison.year(),
            with = comparison.with(),
            calendar_month = comparison.calendar_month(),
            years_with_data = compared.region().len(),
            "years compared"
        );

        Ok(compared)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::features::zones::app::testing::{FakeZoneRepository, RepositoryCall, a_reading};

    fn comparison() -> YearComparison {
        YearComparison::new(2026, 2025, 3).expect("comparison")
    }

    #[tokio::test]
    async fn compares_the_two_years_and_draws_the_region_line_over_all_of_them() {
        let repository = FakeZoneRepository::seeded().with_readings(vec![
            a_reading(1, "2026-03", 60),
            a_reading(1, "2025-03", 55),
            a_reading(2, "2026-03", 30),
            a_reading(2, "2025-03", 70),
            a_reading(1, "2023-03", 40),
            // Another calendar month is neither compared nor loaded.
            a_reading(1, "2026-04", 99),
        ]);
        let use_case = CompareYearsUseCase::new(Arc::new(repository.clone()));

        let compared = use_case.execute(comparison()).await.expect("comparison");

        assert_eq!(
            compared
                .zones()
                .iter()
                .map(|row| (row.zone().slug().as_str(), *row.change()))
                .collect::<Vec<_>>(),
            vec![
                ("kalar", Some(-40)),
                ("chamchamal", Some(5)),
                ("qushtapa", None),
            ]
        );
        assert_eq!(
            compared
                .region()
                .iter()
                .map(|point| (*point.year(), *point.average_dryness()))
                .collect::<Vec<_>>(),
            vec![(2023, 40.0), (2025, 62.5), (2026, 45.0)]
        );
        assert_eq!(
            repository.calls(),
            vec![
                RepositoryCall::FindReadingMonths,
                RepositoryCall::FindReadingsInMonths {
                    months: vec![
                        "2023-03".to_string(),
                        "2025-03".to_string(),
                        "2026-03".to_string(),
                    ],
                },
                RepositoryCall::FindAllZones,
            ]
        );
    }

    #[tokio::test]
    async fn years_without_data_still_list_every_zone() {
        let use_case = CompareYearsUseCase::new(Arc::new(FakeZoneRepository::seeded()));

        let compared = use_case.execute(comparison()).await.expect("comparison");

        assert_eq!(compared.zones().len(), 3);
        assert!(compared.region().is_empty());
    }

    #[tokio::test]
    async fn a_repository_failure_surfaces() {
        let use_case = CompareYearsUseCase::new(Arc::new(FakeZoneRepository::failing()));

        assert!(use_case.execute(comparison()).await.is_err());
    }
}
