use std::sync::Arc;

use super::month_or_latest::month_or_latest;
use crate::{
    app::AppError as GlobalAppError,
    features::zones::{
        app::{AppError, ZoneRepository},
        domain::{Month, ZoneDetail, ZoneSlug},
    },
};

pub struct ViewZoneUseCase {
    repository: Arc<dyn ZoneRepository>,
}

impl ViewZoneUseCase {
    pub fn new(repository: Arc<dyn ZoneRepository>) -> Self {
        Self { repository }
    }

    /// Without a month, shows the latest month that has any zone reading,
    /// the same month the region overview opens on.
    pub async fn execute(
        &self,
        slug: &ZoneSlug,
        month: Option<Month>,
    ) -> Result<ZoneDetail, AppError> {
        let Some(zone) = self.repository.find_zone_by_slug(slug).await? else {
            tracing::info!(zone_slug = slug.as_str(), "view refused: no such zone");

            return Err(GlobalAppError::NotFound.into());
        };

        let month = month_or_latest(self.repository.as_ref(), month).await?;

        let month_readings = self.repository.find_readings_in_months(&[month]).await?;
        let zone_readings = self.repository.find_readings_by_zone(*zone.id()).await?;
        let sub_zones = self.repository.find_sub_zones_by_zone(*zone.id()).await?;

        let sub_zone_ids: Vec<i32> = sub_zones.iter().map(|sub_zone| *sub_zone.id()).collect();

        let sub_zone_readings = self
            .repository
            .find_sub_zone_readings_in_month(&sub_zone_ids, month)
            .await?;

        let detail = ZoneDetail::build(
            zone,
            month,
            &month_readings,
            &zone_readings,
            sub_zones,
            &sub_zone_readings,
        );

        tracing::debug!(
            zone_slug = slug.as_str(),
            month = %String::from(month),
            has_reading = detail.reading().is_some(),
            history_years = detail.history().len(),
            "zone detail built"
        );

        Ok(detail)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::features::zones::app::testing::{
        FakeZoneRepository, RepositoryCall, a_month, a_reading, a_slug, a_sub_zone_reading,
    };

    #[tokio::test]
    async fn shows_the_zone_with_its_rank_sub_zones_and_history() {
        let repository = FakeZoneRepository::seeded()
            .with_readings(vec![
                a_reading(1, "2026-03", 70),
                a_reading(2, "2026-03", 90),
                a_reading(1, "2025-03", 55),
                a_reading(1, "2024-03", 40),
            ])
            .with_sub_zone_readings(vec![
                a_sub_zone_reading(2, "2026-03", 45),
                a_sub_zone_reading(3, "2026-03", 88),
                // Kalar's sub-zone must not show up under Chamchamal.
                a_sub_zone_reading(4, "2026-03", 99),
            ]);
        let use_case = ViewZoneUseCase::new(Arc::new(repository.clone()));

        let detail = use_case
            .execute(&a_slug("chamchamal"), Some(a_month("2026-03")))
            .await
            .expect("detail");

        let ranked = detail.reading().as_ref().expect("reading");

        assert_eq!(*ranked.rank(), 2);
        assert_eq!(*ranked.rank_of(), 2);
        assert_eq!(
            detail
                .sub_zones()
                .iter()
                .map(|row| row.sub_zone().slug().as_str())
                .collect::<Vec<_>>(),
            vec!["sangaw", "aghjalar", "markaz-chamchamal"]
        );
        assert_eq!(
            detail
                .history()
                .iter()
                .map(|point| *point.year())
                .collect::<Vec<_>>(),
            vec![2024, 2025]
        );
        assert_eq!(
            repository.calls(),
            vec![
                RepositoryCall::FindZoneBySlug {
                    slug: "chamchamal".to_string(),
                },
                RepositoryCall::FindReadingsInMonths {
                    months: vec!["2026-03".to_string()],
                },
                RepositoryCall::FindReadingsByZone { zone_id: 1 },
                RepositoryCall::FindSubZonesByZone { zone_id: 1 },
                RepositoryCall::FindSubZoneReadingsInMonth {
                    sub_zone_ids: vec![1, 2, 3],
                    month: "2026-03".to_string(),
                },
            ]
        );
    }

    #[tokio::test]
    async fn an_unknown_zone_is_not_found_and_nothing_else_is_loaded() {
        let repository = FakeZoneRepository::seeded();
        let use_case = ViewZoneUseCase::new(Arc::new(repository.clone()));

        let result = use_case
            .execute(&a_slug("atlantis"), Some(a_month("2026-03")))
            .await;

        assert!(matches!(
            result,
            Err(AppError::GlobalAppError(GlobalAppError::NotFound))
        ));
        assert_eq!(
            repository.calls(),
            vec![RepositoryCall::FindZoneBySlug {
                slug: "atlantis".to_string(),
            }]
        );
    }

    #[tokio::test]
    async fn without_a_month_it_shows_the_latest_month_of_the_whole_region() {
        let repository = FakeZoneRepository::seeded().with_readings(vec![
            a_reading(1, "2026-02", 40),
            // The region's latest month, in which Chamchamal has no reading.
            a_reading(2, "2026-04", 80),
        ]);
        let use_case = ViewZoneUseCase::new(Arc::new(repository));

        let detail = use_case
            .execute(&a_slug("chamchamal"), None)
            .await
            .expect("detail");

        assert_eq!(*detail.month(), a_month("2026-04"));
        assert!(
            detail.reading().is_none(),
            "the zone page shows the month the overview shows, not its own last month"
        );
    }

    #[tokio::test]
    async fn a_known_zone_with_no_data_is_still_shown() {
        let use_case = ViewZoneUseCase::new(Arc::new(FakeZoneRepository::seeded()));

        let detail = use_case
            .execute(&a_slug("kalar"), Some(a_month("2026-03")))
            .await
            .expect("detail");

        assert!(detail.reading().is_none());
        assert!(detail.history().is_empty());
        assert_eq!(detail.sub_zones().len(), 1);
    }

    #[tokio::test]
    async fn a_repository_failure_surfaces() {
        let use_case = ViewZoneUseCase::new(Arc::new(FakeZoneRepository::failing()));

        assert!(use_case.execute(&a_slug("kalar"), None).await.is_err());
    }
}
