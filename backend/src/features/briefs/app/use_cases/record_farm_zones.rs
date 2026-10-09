use std::sync::Arc;

use crate::features::briefs::{
    app::{AppError, BriefRepository},
    domain::FarmZone,
};

pub struct RecordFarmZonesUseCase {
    repository: Arc<dyn BriefRepository>,
}

impl RecordFarmZonesUseCase {
    pub fn new(repository: Arc<dyn BriefRepository>) -> Self {
        Self { repository }
    }

    /// Records which district each farm lies in, replacing what was recorded
    /// for the same farm, and returns how many were recorded. The farms are
    /// not looked up: the job takes its farm ids from the coverage list, and
    /// a district recorded for a farm deleted in between is never read.
    pub async fn execute(&self, zones: Vec<FarmZone>) -> Result<u64, AppError> {
        let zones = FarmZone::batch(zones)?;

        let recorded = self.repository.upsert_farm_zones(&zones).await?;

        tracing::info!(recorded, "farm zones recorded");

        Ok(recorded)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::features::briefs::{
        app::testing::{Call, Fakes, a_farm_zone},
        domain::BriefError,
    };

    #[tokio::test]
    async fn records_every_farm_in_one_call_in_farm_order() {
        let fakes = Fakes::new();
        let use_case = RecordFarmZonesUseCase::new(Arc::new(fakes.clone()));

        let recorded = use_case
            .execute(vec![a_farm_zone(12, "chamchamal"), a_farm_zone(3, "kalar")])
            .await
            .expect("recorded");

        assert_eq!(recorded, 2);
        assert_eq!(fakes.farm_zone(12).as_deref(), Some("chamchamal"));
        assert_eq!(
            fakes.calls(),
            vec![Call::UpsertFarmZones {
                farm_ids: vec![3, 12]
            }],
            "one write for the whole push, so it lands as a whole or not at all"
        );
    }

    #[tokio::test]
    async fn a_farm_recorded_before_moves_to_its_new_zone() {
        let fakes = Fakes::new()
            .with_farm_zone(12, "kalar")
            .with_farm_zone(5, "kifri");
        let use_case = RecordFarmZonesUseCase::new(Arc::new(fakes.clone()));

        use_case
            .execute(vec![a_farm_zone(12, "chamchamal")])
            .await
            .expect("recorded");

        assert_eq!(fakes.farm_zone(12).as_deref(), Some("chamchamal"));
        assert_eq!(
            fakes.farm_zone(5).as_deref(),
            Some("kifri"),
            "a farm the push does not name keeps its zone"
        );
    }

    #[tokio::test]
    async fn a_push_that_names_a_farm_twice_writes_nothing() {
        let fakes = Fakes::new();
        let use_case = RecordFarmZonesUseCase::new(Arc::new(fakes.clone()));

        let result = use_case
            .execute(vec![
                a_farm_zone(12, "chamchamal"),
                a_farm_zone(12, "kalar"),
            ])
            .await;

        assert!(matches!(
            result,
            Err(AppError::Brief(BriefError::DuplicateFarm(12)))
        ));
        assert!(fakes.calls().is_empty());
    }

    #[tokio::test]
    async fn an_empty_push_writes_nothing() {
        let fakes = Fakes::new();
        let use_case = RecordFarmZonesUseCase::new(Arc::new(fakes.clone()));

        assert!(matches!(
            use_case.execute(vec![]).await,
            Err(AppError::Brief(BriefError::FarmCount { .. }))
        ));
        assert!(fakes.calls().is_empty());
    }

    #[tokio::test]
    async fn a_repository_failure_surfaces() {
        let use_case = RecordFarmZonesUseCase::new(Arc::new(Fakes::new().failing()));

        assert!(
            use_case
                .execute(vec![a_farm_zone(12, "kalar")])
                .await
                .is_err()
        );
    }
}
