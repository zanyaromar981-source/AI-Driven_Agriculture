use std::sync::Arc;

use chrono::{DateTime, Utc};

use crate::{
    app::{AppError as GlobalAppError, StaffContext},
    features::fires::{
        app::{AppError, FireRepository},
        domain::{
            Fire, FireCorrection, FireLocation, FireSource, FireStatus, PlaceName, WindDirection,
            ZoneSlug,
        },
    },
};

pub struct CorrectFireInput {
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

pub struct CorrectFireUseCase {
    repository: Arc<dyn FireRepository>,
}

impl CorrectFireUseCase {
    pub fn new(repository: Arc<dyn FireRepository>) -> Self {
        Self { repository }
    }

    /// Replaces every field of a stored fire except its external id with
    /// what a staff member entered. Nothing is read first: whether the fire
    /// exists is decided by the update itself. Unlike a push by the data
    /// job, a correction may carry an older detection time than the stored
    /// one: the person is overriding it on purpose.
    pub async fn execute(
        &self,
        actor: &StaffContext,
        id: i32,
        input: CorrectFireInput,
    ) -> Result<Fire, AppError> {
        let correction = FireCorrection::new(
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

        let Some(stored) = self.repository.update(id, &correction).await? else {
            tracing::info!(
                staff_id = *actor.staff_id(),
                fire_id = id,
                "fire not corrected: no such fire"
            );

            return Err(GlobalAppError::NotFound.into());
        };

        tracing::info!(
            staff_id = *actor.staff_id(),
            fire_id = id,
            external_id = stored.external_id().as_str(),
            status = %String::from(*stored.status()),
            "fire corrected by staff"
        );

        Ok(stored)
    }
}

#[cfg(test)]
mod tests {
    use chrono::Duration;

    use super::*;
    use crate::features::fires::{
        app::testing::{FakeFireRepository, RepositoryCall, a_fire, a_location, a_source, actor},
        domain::FireError,
    };

    fn input() -> CorrectFireInput {
        CorrectFireInput {
            location: a_location(),
            zone_slug: None,
            place_en: None,
            place_ku: None,
            detected_at: Utc::now(),
            area_ha: None,
            wind_kmh: Some(12.0),
            wind_direction: None,
            status: FireStatus::Out,
            farms_within_5km: None,
            farmers_alerted: None,
            source: a_source(),
        }
    }

    #[tokio::test]
    async fn replaces_the_stored_fields_in_one_call_and_keeps_the_external_id() {
        let repository = FakeFireRepository::holding(vec![a_fire(3, FireStatus::Active, 2)]);
        let use_case = CorrectFireUseCase::new(Arc::new(repository.clone()));

        let fire = use_case
            .execute(&actor(), 3, input())
            .await
            .expect("corrected");

        assert_eq!(*fire.id(), Some(3));
        assert_eq!(fire.external_id().as_str(), "firms-3");
        assert_eq!(*fire.status(), FireStatus::Out);
        assert_eq!(
            *fire.area_ha(),
            None,
            "a field the correction leaves out becomes unknown"
        );
        assert_eq!(*fire.zone_slug(), None);
        assert_eq!(
            repository.calls(),
            vec![RepositoryCall::Update { id: 3 }],
            "the fire is not looked up before the write"
        );
    }

    #[tokio::test]
    async fn an_older_detection_time_replaces_a_newer_one() {
        let repository = FakeFireRepository::holding(vec![a_fire(3, FireStatus::Active, 2)]);
        let use_case = CorrectFireUseCase::new(Arc::new(repository.clone()));
        let earlier = Utc::now() - Duration::days(4);

        let fire = use_case
            .execute(
                &actor(),
                3,
                CorrectFireInput {
                    detected_at: earlier,
                    ..input()
                },
            )
            .await
            .expect("corrected");

        assert_eq!(
            *fire.detected_at(),
            earlier,
            "a staff correction is not held back by the newer stored sighting"
        );
        assert_eq!(
            *repository.stored(3).expect("stored").detected_at(),
            earlier
        );
    }

    #[tokio::test]
    async fn a_fire_that_is_not_stored_is_not_found_and_is_not_created() {
        let repository = FakeFireRepository::new();
        let use_case = CorrectFireUseCase::new(Arc::new(repository.clone()));

        let result = use_case.execute(&actor(), 3, input()).await;

        assert!(matches!(
            result,
            Err(AppError::GlobalAppError(GlobalAppError::NotFound))
        ));
        assert!(repository.stored(3).is_none());
    }

    #[tokio::test]
    async fn a_correction_that_breaks_a_rule_is_not_written() {
        let repository = FakeFireRepository::holding(vec![a_fire(3, FireStatus::Active, 2)]);
        let use_case = CorrectFireUseCase::new(Arc::new(repository.clone()));

        let result = use_case
            .execute(
                &actor(),
                3,
                CorrectFireInput {
                    wind_kmh: Some(301.0),
                    ..input()
                },
            )
            .await;

        assert!(matches!(
            result,
            Err(AppError::Fire(FireError::DomainError(_)))
        ));
        assert!(repository.calls().is_empty());
    }

    #[tokio::test]
    async fn a_repository_failure_surfaces() {
        let use_case = CorrectFireUseCase::new(Arc::new(FakeFireRepository::failing()));

        assert!(use_case.execute(&actor(), 3, input()).await.is_err());
    }
}
