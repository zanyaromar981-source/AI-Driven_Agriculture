use std::sync::Arc;

use crate::{
    app::AppError as GlobalAppError,
    features::zones::{
        app::{AppError, ZoneRepository},
        domain::{
            Crop, Dryness, GreennessPctVsNormal, Month, RainPctOfNormal, ReadingSource, WaterNeed,
            ZoneReading, ZoneSlug,
        },
    },
};

pub struct RecordZoneReadingInput {
    pub zone_slug: ZoneSlug,
    pub month: Month,
    pub dryness: Dryness,
    pub rain_pct_of_normal: Option<RainPctOfNormal>,
    pub greenness_pct_vs_normal: Option<GreennessPctVsNormal>,
    pub water_need: Option<WaterNeed>,
    pub nitrogen_hold: bool,
    pub best_crops: Vec<Crop>,
    pub source: ReadingSource,
}

pub struct RecordZoneReadingUseCase {
    repository: Arc<dyn ZoneRepository>,
}

impl RecordZoneReadingUseCase {
    pub fn new(repository: Arc<dyn ZoneRepository>) -> Self {
        Self { repository }
    }

    /// Stores what a data job measured for one zone and month. A second push
    /// for the same zone and month replaces the first, so a job can be run
    /// again safely.
    pub async fn execute(&self, input: RecordZoneReadingInput) -> Result<ZoneReading, AppError> {
        let Some(zone) = self.repository.find_zone_by_slug(&input.zone_slug).await? else {
            tracing::info!(
                zone_slug = input.zone_slug.as_str(),
                "reading refused: no such zone"
            );

            return Err(GlobalAppError::NotFound.into());
        };

        let reading = ZoneReading::new(
            *zone.id(),
            input.month,
            input.dryness,
            input.rain_pct_of_normal,
            input.greenness_pct_vs_normal,
            input.water_need,
            input.nitrogen_hold,
            input.best_crops,
            input.source,
        )?;

        let stored = self.repository.upsert_reading(&reading).await?;

        tracing::info!(
            zone_slug = input.zone_slug.as_str(),
            month = %String::from(input.month),
            dryness = stored.dryness().value(),
            source = stored.source().as_str(),
            "zone reading recorded"
        );

        Ok(stored)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::features::zones::{
        app::testing::{FakeZoneRepository, RepositoryCall, a_dryness, a_month, a_slug, a_source},
        domain::ZoneError,
    };

    fn input(zone_slug: &str, best_crops: Vec<Crop>) -> RecordZoneReadingInput {
        RecordZoneReadingInput {
            zone_slug: a_slug(zone_slug),
            month: a_month("2026-03"),
            dryness: a_dryness(72),
            rain_pct_of_normal: Some(RainPctOfNormal::new(61.5).expect("rain")),
            greenness_pct_vs_normal: Some(GreennessPctVsNormal::new(-18.0).expect("greenness")),
            water_need: Some(WaterNeed::new(80).expect("need")),
            nitrogen_hold: true,
            best_crops,
            source: a_source(),
        }
    }

    #[tokio::test]
    async fn stores_the_reading_under_the_zone_the_slug_names() {
        let repository = FakeZoneRepository::seeded();
        let use_case = RecordZoneReadingUseCase::new(Arc::new(repository.clone()));

        let stored = use_case
            .execute(input("kalar", vec![Crop::Barley, Crop::Wheat]))
            .await
            .expect("stored");

        assert_eq!(*stored.zone_id(), 2);
        assert_eq!(stored.dryness().value(), 72);
        assert_eq!(stored.best_crops(), &vec![Crop::Barley, Crop::Wheat]);
        assert!(stored.id().is_some(), "the stored reading is returned");
        assert_eq!(
            repository.calls(),
            vec![
                RepositoryCall::FindZoneBySlug {
                    slug: "kalar".to_string(),
                },
                RepositoryCall::UpsertReading {
                    zone_id: 2,
                    month: "2026-03".to_string(),
                },
            ]
        );
    }

    #[tokio::test]
    async fn an_unknown_zone_is_not_found_and_nothing_is_written() {
        let repository = FakeZoneRepository::seeded();
        let use_case = RecordZoneReadingUseCase::new(Arc::new(repository.clone()));

        let result = use_case.execute(input("atlantis", vec![])).await;

        assert!(matches!(
            result,
            Err(AppError::GlobalAppError(GlobalAppError::NotFound))
        ));
        assert!(!repository.wrote(), "a refused reading must not write");
    }

    #[tokio::test]
    async fn a_crop_ranked_twice_is_refused_and_nothing_is_written() {
        let repository = FakeZoneRepository::seeded();
        let use_case = RecordZoneReadingUseCase::new(Arc::new(repository.clone()));

        let result = use_case
            .execute(input("kalar", vec![Crop::Wheat, Crop::Wheat]))
            .await;

        assert!(matches!(
            result,
            Err(AppError::Zone(ZoneError::RepeatedCrop(_)))
        ));
        assert!(!repository.wrote());
    }

    #[tokio::test]
    async fn a_repository_failure_surfaces_instead_of_recording() {
        let repository = FakeZoneRepository::failing();
        let use_case = RecordZoneReadingUseCase::new(Arc::new(repository.clone()));

        assert!(use_case.execute(input("kalar", vec![])).await.is_err());
        assert!(!repository.wrote());
    }
}
