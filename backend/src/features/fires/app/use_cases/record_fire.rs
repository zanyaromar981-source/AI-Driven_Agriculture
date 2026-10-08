use std::sync::Arc;

use chrono::{DateTime, Utc};

use crate::features::fires::{
    app::{AppError, FireRepository},
    domain::{
        ExternalId, Fire, FireLocation, FireSource, FireStatus, PlaceName, WindDirection, ZoneSlug,
    },
};

pub struct RecordFireInput {
    pub external_id: ExternalId,
    pub location: FireLocation,
    pub zone_slug: Option<ZoneSlug>,
    pub place_en: Option<PlaceName>,
    pub place_ku: Option<PlaceName>,
    pub detected_at: DateTime<Utc>,
    pub area_ha: Option<f64>,
    pub wind_kmh: Option<f64>,
    pub wind_direction: Option<WindDirection>,
    pub status: FireStatus,
    pub farms_within_5km: Option<i32>,
    pub farmers_alerted: Option<i32>,
    pub source: FireSource,
}

pub struct RecordFireUseCase {
    repository: Arc<dyn FireRepository>,
}

impl RecordFireUseCase {
    pub fn new(repository: Arc<dyn FireRepository>) -> Self {
        Self { repository }
    }

    /// Stores what the data job says about one fire. The push describes the
    /// whole fire: a field it leaves out is stored as unknown, it does not
    /// keep the value of an earlier push.
    pub async fn execute(&self, input: RecordFireInput) -> Result<Fire, AppError> {
        let fire = Fire::new(
            input.external_id,
            input.location,
            input.zone_slug,
            input.place_en,
            input.place_ku,
            input.detected_at,
            input.area_ha,
            input.wind_kmh,
            input.wind_direction,
            input.status,
            input.farms_within_5km,
            input.farmers_alerted,
            input.source,
        )?;

        let stored = self.repository.upsert(&fire).await?;

        tracing::info!(
            fire_id = stored.id().unwrap_or_default(),
            external_id = stored.external_id().as_str(),
            status = %String::from(*stored.status()),
            "fire recorded"
        );

        Ok(stored)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::features::fires::{
        app::testing::{FakeFireRepository, RepositoryCall, a_location, a_source, an_external_id},
        domain::FireError,
    };

    fn input() -> RecordFireInput {
        RecordFireInput {
            external_id: an_external_id("firms-42"),
            location: a_location(),
            zone_slug: None,
            place_en: None,
            place_ku: None,
            detected_at: Utc::now(),
            area_ha: Some(3.5),
            wind_kmh: Some(22.0),
            wind_direction: Some(WindDirection::Se),
            status: FireStatus::Spreading,
            farms_within_5km: Some(6),
            farmers_alerted: Some(11),
            source: a_source(),
        }
    }

    #[tokio::test]
    async fn stores_the_fire_under_its_external_id() {
        let repository = FakeFireRepository::new();
        let use_case = RecordFireUseCase::new(Arc::new(repository.clone()));

        let fire = use_case.execute(input()).await.expect("fire");

        assert!(fire.id().is_some(), "the answer is the stored fire");
        assert_eq!(*fire.status(), FireStatus::Spreading);
        assert_eq!(
            repository.calls(),
            vec![RepositoryCall::Upsert {
                external_id: "firms-42".to_string(),
            }]
        );
    }

    #[tokio::test]
    async fn the_numbers_are_stored_as_they_were_pushed() {
        let use_case = RecordFireUseCase::new(Arc::new(FakeFireRepository::new()));

        let fire = use_case.execute(input()).await.expect("fire");

        assert_eq!(*fire.area_ha(), Some(3.5));
        assert_eq!(*fire.wind_kmh(), Some(22.0));
        assert_eq!(*fire.farms_within_5km(), Some(6));
        assert_eq!(*fire.farmers_alerted(), Some(11));
    }

    #[tokio::test]
    async fn a_number_that_was_not_pushed_stays_unknown() {
        let use_case = RecordFireUseCase::new(Arc::new(FakeFireRepository::new()));

        let fire = use_case
            .execute(RecordFireInput {
                area_ha: None,
                farms_within_5km: None,
                ..input()
            })
            .await
            .expect("fire");

        assert_eq!(*fire.area_ha(), None, "unknown must not become 0");
        assert_eq!(*fire.farms_within_5km(), None);
    }

    #[tokio::test]
    async fn a_fire_that_breaks_a_rule_is_not_written() {
        let repository = FakeFireRepository::new();
        let use_case = RecordFireUseCase::new(Arc::new(repository.clone()));

        let result = use_case
            .execute(RecordFireInput {
                wind_kmh: Some(301.0),
                ..input()
            })
            .await;

        assert!(matches!(
            result,
            Err(AppError::Fire(FireError::DomainError(_)))
        ));
        assert!(repository.calls().is_empty());
    }

    #[tokio::test]
    async fn a_repository_failure_surfaces() {
        let use_case = RecordFireUseCase::new(Arc::new(FakeFireRepository::failing()));

        assert!(use_case.execute(input()).await.is_err());
    }
}
