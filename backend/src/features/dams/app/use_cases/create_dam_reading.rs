use std::sync::Arc;

use crate::{
    app::StaffContext,
    features::dams::{
        app::{AppError, DamRepository, use_cases::RecordDamReadingInput},
        domain::DamReading,
    },
};

pub struct CreateDamReadingUseCase {
    repository: Arc<dyn DamRepository>,
}

impl CreateDamReadingUseCase {
    pub fn new(repository: Arc<dyn DamRepository>) -> Self {
        Self { repository }
    }

    /// Stores a reading a staff member typed in for a day the dam has none
    /// for. Whether the day is free is not looked up first: the unique index
    /// answers that when the reading is stored, so of two copies sent at the
    /// same moment exactly one is created.
    pub async fn execute(
        &self,
        actor: &StaffContext,
        input: RecordDamReadingInput,
    ) -> Result<DamReading, AppError> {
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

        let created = self
            .repository
            .create_reading(&reading)
            .await?
            .ok_or_else(|| AppError::ReadingAlreadyExists {
                slug: String::from(&input.slug),
                day: input.day,
            })?;

        tracing::info!(
            staff_id = *actor.staff_id(),
            dam = input.slug.as_str(),
            day = %created.day(),
            pct_full = created.pct_full().value(),
            source = created.source().as_str(),
            "dam reading created from the dashboard"
        );

        Ok(created)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::features::dams::{
        app::testing::{
            FakeDamRepository, RepositoryCall, a_reading, darbandikhan, day, dukan, slug, staff,
        },
        domain::{DamError, PercentFull, ReadingSource},
    };

    fn input(dam: &str, volume_bn_m3: Option<f64>) -> RecordDamReadingInput {
        RecordDamReadingInput {
            slug: slug(dam),
            day: day(2026, 10, 1),
            pct_full: PercentFull::new(38.5).expect("percent"),
            volume_bn_m3,
            lake_area_km2: Some(140.0),
            farm_supply_bn_m3: Some(0.9),
            source: ReadingSource::new("gauge read by hand".to_string()).expect("source"),
        }
    }

    fn repository() -> FakeDamRepository {
        FakeDamRepository::holding(vec![dukan(), darbandikhan()], vec![])
    }

    #[tokio::test]
    async fn creates_the_reading_under_the_dam_the_slug_names() {
        let repository = repository();
        let use_case = CreateDamReadingUseCase::new(Arc::new(repository.clone()));

        let reading = use_case
            .execute(&staff(), input("darbandikhan", Some(1.2)))
            .await
            .expect("reading");

        assert_eq!(*reading.dam_id(), 2);
        assert_eq!(*reading.day(), day(2026, 10, 1));
        assert_eq!(*reading.volume_bn_m3(), Some(1.2));
        assert_eq!(reading.source().as_str(), "gauge read by hand");
        assert!(reading.id().is_some(), "the stored reading comes back");
        assert_eq!(
            repository.calls(),
            vec![
                RepositoryCall::FindBySlug {
                    slug: "darbandikhan".to_string()
                },
                RepositoryCall::CreateReading {
                    dam_id: 2,
                    day: day(2026, 10, 1),
                },
            ],
            "the create-only write decides: no reading is looked up before it"
        );
    }

    #[tokio::test]
    async fn a_day_that_already_has_a_reading_is_refused_and_left_as_it_was() {
        let repository = FakeDamRepository::holding(
            vec![dukan()],
            vec![a_reading(&dukan(), day(2026, 10, 1), 61.0)],
        );
        let use_case = CreateDamReadingUseCase::new(Arc::new(repository.clone()));

        let result = use_case.execute(&staff(), input("dukan", None)).await;

        assert!(matches!(
            result,
            Err(AppError::ReadingAlreadyExists { slug, day: on })
                if slug == "dukan" && on == day(2026, 10, 1)
        ));
        assert!(
            !repository
                .calls()
                .iter()
                .any(|call| matches!(call, RepositoryCall::UpsertReading { .. })),
            "a create must never fall back to replacing"
        );
    }

    #[tokio::test]
    async fn the_same_day_on_another_dam_is_free() {
        let repository = FakeDamRepository::holding(
            vec![dukan(), darbandikhan()],
            vec![a_reading(&dukan(), day(2026, 10, 1), 61.0)],
        );
        let use_case = CreateDamReadingUseCase::new(Arc::new(repository));

        assert!(
            use_case
                .execute(&staff(), input("darbandikhan", None))
                .await
                .is_ok()
        );
    }

    #[tokio::test]
    async fn an_unknown_dam_is_not_found_and_nothing_is_written() {
        let repository = repository();
        let use_case = CreateDamReadingUseCase::new(Arc::new(repository.clone()));

        let result = use_case.execute(&staff(), input("mosul", None)).await;

        assert!(matches!(result, Err(AppError::DamNotFound(slug)) if slug == "mosul"));
        assert_eq!(
            repository.calls(),
            vec![RepositoryCall::FindBySlug {
                slug: "mosul".to_string()
            }]
        );
    }

    #[tokio::test]
    async fn a_volume_over_the_dams_capacity_is_refused_as_on_ingest() {
        let repository = repository();
        let use_case = CreateDamReadingUseCase::new(Arc::new(repository.clone()));

        // 5 billion m3 does not fit in Darbandikhan (3.0).
        let result = use_case
            .execute(&staff(), input("darbandikhan", Some(5.0)))
            .await;

        assert!(matches!(
            result,
            Err(AppError::Dam(DamError::VolumeOverCapacity { .. }))
        ));
        assert!(
            !repository
                .calls()
                .iter()
                .any(|call| matches!(call, RepositoryCall::CreateReading { .. })),
            "the refused reading must not be written"
        );
    }

    #[tokio::test]
    async fn a_repository_failure_surfaces() {
        let use_case = CreateDamReadingUseCase::new(Arc::new(FakeDamRepository::failing()));

        assert!(
            use_case
                .execute(&staff(), input("dukan", None))
                .await
                .is_err()
        );
    }
}
