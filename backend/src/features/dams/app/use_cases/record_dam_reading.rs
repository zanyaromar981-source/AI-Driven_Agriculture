use std::sync::Arc;

use chrono::NaiveDate;

use crate::features::dams::{
    app::{AppError, DamRepository},
    domain::{DamReading, DamSlug, PercentFull, ReadingSource},
};

pub struct RecordDamReadingInput {
    pub slug: DamSlug,
    pub day: NaiveDate,
    pub pct_full: PercentFull,
    pub volume_bn_m3: Option<f64>,
    pub lake_area_km2: Option<f64>,
    pub farm_supply_bn_m3: Option<f64>,
    pub source: ReadingSource,
}

pub struct RecordDamReadingUseCase {
    repository: Arc<dyn DamRepository>,
}

impl RecordDamReadingUseCase {
    pub fn new(repository: Arc<dyn DamRepository>) -> Self {
        Self { repository }
    }

    /// Stores what a data job measured for one dam on one day. Sending the
    /// same day again replaces the earlier reading.
    pub async fn execute(&self, input: RecordDamReadingInput) -> Result<DamReading, AppError> {
        let dam = self
            .repository
            .find_by_slug(&input.slug)
            .await?
            .ok_or_else(|| AppError::DamNotFound(String::from(&input.slug)))?;

        let reading = DamReading::new(
            &dam,
            input.day,
            input.pct_full,
            input.volume_bn_m3,
            input.lake_area_km2,
            input.farm_supply_bn_m3,
            input.source,
        )?;

        let recorded = self.repository.upsert_reading(&reading).await?;

        tracing::info!(
            dam = input.slug.as_str(),
            day = %recorded.day(),
            pct_full = recorded.pct_full().value(),
            source = recorded.source().as_str(),
            "dam reading recorded"
        );

        Ok(recorded)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::features::dams::{
        app::testing::{FakeDamRepository, RepositoryCall, darbandikhan, day, dukan, slug},
        domain::DamError,
    };

    fn input(dam: &str, volume_bn_m3: Option<f64>) -> RecordDamReadingInput {
        RecordDamReadingInput {
            slug: slug(dam),
            day: day(2026, 10, 1),
            pct_full: PercentFull::new(38.5).expect("percent"),
            volume_bn_m3,
            lake_area_km2: Some(140.0),
            farm_supply_bn_m3: Some(0.9),
            source: ReadingSource::new("sentinel-2".to_string()).expect("source"),
        }
    }

    fn repository() -> FakeDamRepository {
        FakeDamRepository::holding(vec![dukan(), darbandikhan()], vec![])
    }

    #[tokio::test]
    async fn records_the_reading_under_the_dam_the_slug_names() {
        let repository = repository();
        let use_case = RecordDamReadingUseCase::new(Arc::new(repository.clone()));

        let reading = use_case
            .execute(input("darbandikhan", Some(1.2)))
            .await
            .expect("reading");

        assert_eq!(*reading.dam_id(), 2);
        assert_eq!(*reading.day(), day(2026, 10, 1));
        assert_eq!(reading.pct_full().value(), 38.5);
        assert_eq!(*reading.volume_bn_m3(), Some(1.2));
        assert_eq!(reading.source().as_str(), "sentinel-2");
        assert!(reading.id().is_some(), "the stored reading comes back");
        assert_eq!(
            repository.calls(),
            vec![
                RepositoryCall::FindBySlug {
                    slug: "darbandikhan".to_string()
                },
                RepositoryCall::UpsertReading {
                    dam_id: 2,
                    day: day(2026, 10, 1),
                },
            ]
        );
    }

    #[tokio::test]
    async fn an_unknown_dam_is_not_found_and_nothing_is_written() {
        let repository = repository();
        let use_case = RecordDamReadingUseCase::new(Arc::new(repository.clone()));

        let result = use_case.execute(input("mosul", None)).await;

        assert!(matches!(result, Err(AppError::DamNotFound(slug)) if slug == "mosul"));
        assert_eq!(
            repository.calls(),
            vec![RepositoryCall::FindBySlug {
                slug: "mosul".to_string()
            }],
            "a reading for a dam we do not know must not be stored"
        );
    }

    #[tokio::test]
    async fn the_capacity_checked_is_that_of_the_named_dam() {
        let repository = repository();
        let use_case = RecordDamReadingUseCase::new(Arc::new(repository.clone()));

        // 5 billion m3 fits in Dukan (6.97) but not in Darbandikhan (3.0).
        let in_dukan = use_case.execute(input("dukan", Some(5.0))).await;
        let in_darbandikhan = use_case.execute(input("darbandikhan", Some(5.0))).await;

        assert!(in_dukan.is_ok());
        assert!(matches!(
            in_darbandikhan,
            Err(AppError::Dam(DamError::VolumeOverCapacity { .. }))
        ));
        assert_eq!(
            repository
                .calls()
                .iter()
                .filter(|call| matches!(call, RepositoryCall::UpsertReading { .. }))
                .count(),
            1,
            "the refused reading must not be written"
        );
    }

    #[tokio::test]
    async fn a_repository_failure_surfaces() {
        let use_case = RecordDamReadingUseCase::new(Arc::new(FakeDamRepository::failing()));

        assert!(use_case.execute(input("dukan", None)).await.is_err());
    }
}
