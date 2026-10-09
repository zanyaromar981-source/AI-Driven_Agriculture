use std::sync::Arc;

use crate::{
    app::StaffContext,
    features::fires::{
        app::{AppError, FireRepository, use_cases::RecordFireInput},
        domain::{Fire, FireError},
    },
};

pub struct CreateFireUseCase {
    repository: Arc<dyn FireRepository>,
}

impl CreateFireUseCase {
    pub fn new(repository: Arc<dyn FireRepository>) -> Self {
        Self { repository }
    }

    /// Stores a fire a staff member enters by hand. Nothing is read first:
    /// whether the external id is free is decided by the insert itself, so
    /// of two creates sent at the same moment exactly one succeeds.
    pub async fn execute(
        &self,
        actor: &StaffContext,
        input: RecordFireInput,
    ) -> Result<Fire, AppError> {
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

        let Some(stored) = self.repository.create(&fire).await? else {
            tracing::info!(
                staff_id = *actor.staff_id(),
                external_id = fire.external_id().as_str(),
                "fire not created: the external id is already stored"
            );

            return Err(FireError::AlreadyExists.into());
        };

        tracing::info!(
            staff_id = *actor.staff_id(),
            fire_id = stored.id().unwrap_or_default(),
            external_id = stored.external_id().as_str(),
            status = %String::from(*stored.status()),
            "fire created by staff"
        );

        Ok(stored)
    }
}

#[cfg(test)]
mod tests {
    use chrono::Utc;

    use super::*;
    use crate::features::fires::{
        app::testing::{
            FakeFireRepository, RepositoryCall, a_fire, a_location, a_source, actor, an_external_id,
        },
        domain::{FireStatus, WindDirection},
    };

    fn input(external_id: &str) -> RecordFireInput {
        RecordFireInput {
            external_id: an_external_id(external_id),
            location: a_location(),
            zone_slug: None,
            place_en: None,
            place_ku: None,
            detected_at: Utc::now(),
            area_ha: Some(3.5),
            wind_kmh: None,
            wind_direction: Some(WindDirection::Se),
            status: FireStatus::Spreading,
            farms_within_5km: None,
            farmers_alerted: Some(11),
            source: a_source(),
        }
    }

    #[tokio::test]
    async fn stores_a_new_fire_in_one_call_without_reading_first() {
        let repository = FakeFireRepository::new();
        let use_case = CreateFireUseCase::new(Arc::new(repository.clone()));

        let fire = use_case
            .execute(&actor(), input("manual-1"))
            .await
            .expect("fire");

        assert!(fire.id().is_some(), "the answer is the stored fire");
        assert_eq!(*fire.area_ha(), Some(3.5));
        assert_eq!(*fire.wind_kmh(), None, "unknown must not become 0");
        assert_eq!(
            repository.calls(),
            vec![RepositoryCall::Create {
                external_id: "manual-1".to_string(),
            }]
        );
    }

    #[tokio::test]
    async fn an_external_id_that_is_already_stored_is_refused_and_nothing_changes() {
        let repository = FakeFireRepository::holding(vec![a_fire(1, FireStatus::Out, 5)]);
        let use_case = CreateFireUseCase::new(Arc::new(repository.clone()));

        let result = use_case.execute(&actor(), input("firms-1")).await;

        assert!(matches!(
            result,
            Err(AppError::Fire(FireError::AlreadyExists))
        ));
        assert_eq!(
            *repository.stored(1).expect("stored").status(),
            FireStatus::Out,
            "a create must never replace the stored fire"
        );
    }

    #[tokio::test]
    async fn a_fire_that_breaks_a_rule_is_not_written() {
        let repository = FakeFireRepository::new();
        let use_case = CreateFireUseCase::new(Arc::new(repository.clone()));

        let result = use_case
            .execute(
                &actor(),
                RecordFireInput {
                    area_ha: Some(-1.0),
                    ..input("manual-1")
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
        let use_case = CreateFireUseCase::new(Arc::new(FakeFireRepository::failing()));

        assert!(use_case.execute(&actor(), input("manual-1")).await.is_err());
    }
}
