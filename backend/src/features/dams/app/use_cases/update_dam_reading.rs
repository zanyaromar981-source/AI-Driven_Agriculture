use std::sync::Arc;

use crate::{
    app::StaffContext,
    features::dams::{
        app::{AppError, DamRepository, use_cases::RecordDamReadingInput},
        domain::DamReading,
    },
};

pub struct UpdateDamReadingUseCase {
    repository: Arc<dyn DamRepository>,
}

impl UpdateDamReadingUseCase {
    pub fn new(repository: Arc<dyn DamRepository>) -> Self {
        Self { repository }
    }

    /// Replaces the stored reading of one dam and day with what a staff
    /// member corrected it to. Whether there is one is not looked up first:
    /// the update says how many rows it touched, and none means not found.
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

        let updated = self
            .repository
            .update_reading(&reading)
            .await?
            .ok_or_else(|| AppError::ReadingNotFound {
                slug: String::from(&input.slug),
                day: input.day,
            })?;

        tracing::info!(
            staff_id = *actor.staff_id(),
            dam = input.slug.as_str(),
            day = %updated.day(),
            pct_full = updated.pct_full().value(),
            source = updated.source().as_str(),
            "dam reading updated from the dashboard"
        );

        Ok(updated)
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
            lake_area_km2: None,
            farm_supply_bn_m3: None,
            source: ReadingSource::new("corrected by hand".to_string()).expect("source"),
        }
    }

    fn repository() -> FakeDamRepository {
        FakeDamRepository::holding(
            vec![dukan(), darbandikhan()],
            vec![a_reading(&dukan(), day(2026, 10, 1), 61.0)],
        )
    }

    #[tokio::test]
    async fn replaces_the_reading_of_that_dam_and_day() {
        let repository = repository();
        let use_case = UpdateDamReadingUseCase::new(Arc::new(repository.clone()));

        let reading = use_case
            .execute(&staff(), input("dukan", Some(2.5)))
            .await
            .expect("reading");

        assert_eq!(reading.pct_full().value(), 38.5);
        assert_eq!(*reading.volume_bn_m3(), Some(2.5));
        assert_eq!(reading.source().as_str(), "corrected by hand");
        assert_eq!(
            repository.calls(),
            vec![
                RepositoryCall::FindBySlug {
                    slug: "dukan".to_string()
                },
                RepositoryCall::UpdateReading {
                    dam_id: 1,
                    day: day(2026, 10, 1),
                },
            ],
            "the update-only write decides: no reading is looked up before it"
        );
    }

    #[tokio::test]
    async fn a_day_without_a_reading_is_not_found_and_none_is_created() {
        let repository = repository();
        let use_case = UpdateDamReadingUseCase::new(Arc::new(repository.clone()));

        // The reading is Dukan's: Darbandikhan has none for the day.
        let result = use_case
            .execute(&staff(), input("darbandikhan", None))
            .await;

        assert!(matches!(
            result,
            Err(AppError::ReadingNotFound { slug, day: on })
                if slug == "darbandikhan" && on == day(2026, 10, 1)
        ));
        assert!(
            !repository.calls().iter().any(|call| matches!(
                call,
                RepositoryCall::UpsertReading { .. } | RepositoryCall::CreateReading { .. }
            )),
            "an update must never fall back to creating"
        );
    }

    #[tokio::test]
    async fn an_unknown_dam_is_not_found_and_nothing_is_written() {
        let repository = repository();
        let use_case = UpdateDamReadingUseCase::new(Arc::new(repository.clone()));

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
        let use_case = UpdateDamReadingUseCase::new(Arc::new(repository.clone()));

        // Dukan holds 6.97 billion m3.
        let result = use_case.execute(&staff(), input("dukan", Some(9.0))).await;

        assert!(matches!(
            result,
            Err(AppError::Dam(DamError::VolumeOverCapacity { .. }))
        ));
        assert!(
            !repository
                .calls()
                .iter()
                .any(|call| matches!(call, RepositoryCall::UpdateReading { .. })),
            "the refused reading must not be written"
        );
    }

    #[tokio::test]
    async fn a_repository_failure_surfaces() {
        let use_case = UpdateDamReadingUseCase::new(Arc::new(FakeDamRepository::failing()));

        assert!(
            use_case
                .execute(&staff(), input("dukan", None))
                .await
                .is_err()
        );
    }
}
