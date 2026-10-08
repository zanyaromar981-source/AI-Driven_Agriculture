use std::sync::Arc;

use crate::{
    app::AppError as GlobalAppError,
    features::zones::{
        app::{AppError, ZoneRepository},
        domain::{Dryness, Month, SubZoneReading, ZoneSlug},
    },
};

pub struct RecordSubZoneReadingInput {
    pub zone_slug: ZoneSlug,
    pub sub_zone_slug: ZoneSlug,
    pub month: Month,
    pub dryness: Dryness,
}

pub struct RecordSubZoneReadingUseCase {
    repository: Arc<dyn ZoneRepository>,
}

impl RecordSubZoneReadingUseCase {
    pub fn new(repository: Arc<dyn ZoneRepository>) -> Self {
        Self { repository }
    }

    /// Stores the dryness a data job measured for one sub-zone and month,
    /// replacing an earlier push for the same sub-zone and month.
    pub async fn execute(
        &self,
        input: RecordSubZoneReadingInput,
    ) -> Result<SubZoneReading, AppError> {
        let Some(zone) = self.repository.find_zone_by_slug(&input.zone_slug).await? else {
            tracing::info!(
                zone_slug = input.zone_slug.as_str(),
                "sub-zone reading refused: no such zone"
            );

            return Err(GlobalAppError::NotFound.into());
        };

        let Some(sub_zone) = self
            .repository
            .find_sub_zone_by_slug(*zone.id(), &input.sub_zone_slug)
            .await?
        else {
            tracing::info!(
                zone_slug = input.zone_slug.as_str(),
                sub_zone_slug = input.sub_zone_slug.as_str(),
                "sub-zone reading refused: no such sub-zone in this zone"
            );

            return Err(GlobalAppError::NotFound.into());
        };

        let reading = SubZoneReading::new(*sub_zone.id(), input.month, input.dryness);

        let stored = self.repository.upsert_sub_zone_reading(&reading).await?;

        tracing::info!(
            zone_slug = input.zone_slug.as_str(),
            sub_zone_slug = input.sub_zone_slug.as_str(),
            month = %String::from(input.month),
            dryness = stored.dryness().value(),
            "sub-zone reading recorded"
        );

        Ok(stored)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::features::zones::app::testing::{
        FakeZoneRepository, RepositoryCall, a_dryness, a_month, a_slug,
    };

    fn input(zone_slug: &str, sub_zone_slug: &str) -> RecordSubZoneReadingInput {
        RecordSubZoneReadingInput {
            zone_slug: a_slug(zone_slug),
            sub_zone_slug: a_slug(sub_zone_slug),
            month: a_month("2026-03"),
            dryness: a_dryness(64),
        }
    }

    #[tokio::test]
    async fn stores_the_reading_under_the_sub_zone_of_that_zone() {
        let repository = FakeZoneRepository::seeded();
        let use_case = RecordSubZoneReadingUseCase::new(Arc::new(repository.clone()));

        let stored = use_case
            .execute(input("chamchamal", "sangaw"))
            .await
            .expect("stored");

        assert_eq!(*stored.sub_zone_id(), 3);
        assert_eq!(stored.dryness().value(), 64);
        assert_eq!(
            repository.calls(),
            vec![
                RepositoryCall::FindZoneBySlug {
                    slug: "chamchamal".to_string(),
                },
                RepositoryCall::FindSubZoneBySlug {
                    zone_id: 1,
                    slug: "sangaw".to_string(),
                },
                RepositoryCall::UpsertSubZoneReading {
                    sub_zone_id: 3,
                    month: "2026-03".to_string(),
                },
            ]
        );
    }

    #[tokio::test]
    async fn an_unknown_zone_is_not_found_and_nothing_is_written() {
        let repository = FakeZoneRepository::seeded();
        let use_case = RecordSubZoneReadingUseCase::new(Arc::new(repository.clone()));

        let result = use_case.execute(input("atlantis", "sangaw")).await;

        assert!(matches!(
            result,
            Err(AppError::GlobalAppError(GlobalAppError::NotFound))
        ));
        assert!(!repository.wrote());
    }

    #[tokio::test]
    async fn a_sub_zone_of_another_zone_is_not_found() {
        let repository = FakeZoneRepository::seeded();
        let use_case = RecordSubZoneReadingUseCase::new(Arc::new(repository.clone()));

        let result = use_case.execute(input("kalar", "sangaw")).await;

        assert!(
            matches!(
                result,
                Err(AppError::GlobalAppError(GlobalAppError::NotFound))
            ),
            "sangaw belongs to chamchamal, so it does not exist under kalar"
        );
        assert!(!repository.wrote());
    }

    #[tokio::test]
    async fn a_repository_failure_surfaces_instead_of_recording() {
        let repository = FakeZoneRepository::failing();
        let use_case = RecordSubZoneReadingUseCase::new(Arc::new(repository.clone()));

        assert!(
            use_case
                .execute(input("chamchamal", "sangaw"))
                .await
                .is_err()
        );
        assert!(!repository.wrote());
    }
}
