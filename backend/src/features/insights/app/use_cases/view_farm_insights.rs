use std::sync::Arc;

use crate::{
    app::{AppError as GlobalAppError, AuthContext},
    features::insights::{
        app::{AppError, FarmOwnership, InsightRepository},
        domain::FarmInsight,
    },
};

pub struct ViewFarmInsightsUseCase {
    repository: Arc<dyn InsightRepository>,
    ownership: Arc<dyn FarmOwnership>,
}

impl ViewFarmInsightsUseCase {
    pub fn new(repository: Arc<dyn InsightRepository>, ownership: Arc<dyn FarmOwnership>) -> Self {
        Self {
            repository,
            ownership,
        }
    }

    /// Returns the farm's current readings in the order the topics are
    /// declared. A farm with no readings yet gives an empty list. A farm of
    /// another phone is answered like one that does not exist.
    pub async fn execute(
        &self,
        auth_context: &AuthContext,
        farm_id: i32,
    ) -> Result<Vec<FarmInsight>, AppError> {
        if !self
            .ownership
            .is_owned_by(farm_id, auth_context.user().phone())
            .await?
        {
            tracing::info!(farm_id, "insights refused: no such farm for this owner");

            return Err(GlobalAppError::NotFound.into());
        }

        let mut insights = self.repository.find_all_by_farm(farm_id).await?;
        insights.sort_by_key(|insight| *insight.topic());

        tracing::debug!(farm_id, returned = insights.len(), "farm insights viewed");

        Ok(insights)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::features::insights::{
        app::testing::{Call, FARM_ID, Fakes, OWNER, an_insight, auth_context},
        domain::Topic,
    };

    fn use_case(fakes: &Fakes) -> ViewFarmInsightsUseCase {
        ViewFarmInsightsUseCase::new(Arc::new(fakes.clone()), Arc::new(fakes.clone()))
    }

    #[tokio::test]
    async fn returns_the_readings_of_a_farm_the_user_owns_in_topic_order() {
        let fakes = Fakes::new()
            .with_owned_farm()
            .with_stored(an_insight(FARM_ID, Topic::Weather, 8))
            .with_stored(an_insight(FARM_ID, Topic::SurfaceWater, 7))
            .with_stored(an_insight(FARM_ID, Topic::Rain, 6));

        let insights = use_case(&fakes)
            .execute(&auth_context(), FARM_ID)
            .await
            .expect("insights");

        let topics: Vec<Topic> = insights.iter().map(|insight| *insight.topic()).collect();

        assert_eq!(
            topics,
            vec![Topic::SurfaceWater, Topic::Rain, Topic::Weather]
        );
    }

    #[tokio::test]
    async fn ownership_is_checked_for_the_signed_in_phone_before_anything_is_read() {
        let fakes = Fakes::new().with_owned_farm();

        use_case(&fakes)
            .execute(&auth_context(), FARM_ID)
            .await
            .expect("insights");

        assert_eq!(
            fakes.calls(),
            vec![
                Call::IsOwnedBy {
                    farm_id: FARM_ID,
                    phone: OWNER.to_string(),
                },
                Call::FindAllByFarm { farm_id: FARM_ID },
            ]
        );
    }

    #[tokio::test]
    async fn a_farm_with_no_readings_yet_is_an_empty_list_not_an_error() {
        let fakes = Fakes::new().with_owned_farm();

        let insights = use_case(&fakes)
            .execute(&auth_context(), FARM_ID)
            .await
            .expect("insights");

        assert!(insights.is_empty());
    }

    #[tokio::test]
    async fn is_not_found_when_the_user_does_not_own_the_farm_and_reads_nothing() {
        let fakes = Fakes::new()
            .with_owned_farm()
            .with_stored(an_insight(99, Topic::Rain, 6));

        let result = use_case(&fakes).execute(&auth_context(), 99).await;

        assert!(matches!(
            result,
            Err(AppError::GlobalAppError(GlobalAppError::NotFound))
        ));
        assert!(
            !fakes.calls().contains(&Call::FindAllByFarm { farm_id: 99 }),
            "readings of another farmer's farm must not even be loaded"
        );
    }

    #[tokio::test]
    async fn readings_of_other_farms_are_never_mixed_in() {
        let fakes = Fakes::new()
            .with_owned_farm()
            .with_stored(an_insight(FARM_ID, Topic::Soil, 5))
            .with_stored(an_insight(99, Topic::Rain, 6));

        let insights = use_case(&fakes)
            .execute(&auth_context(), FARM_ID)
            .await
            .expect("insights");

        assert_eq!(insights.len(), 1);
        assert_eq!(*insights[0].farm_id(), FARM_ID);
    }

    #[tokio::test]
    async fn a_failed_ownership_check_surfaces_instead_of_reading() {
        let fakes = Fakes::new().with_owned_farm().failing();

        let result = use_case(&fakes).execute(&auth_context(), FARM_ID).await;

        assert!(result.is_err());
        assert!(
            !fakes
                .calls()
                .contains(&Call::FindAllByFarm { farm_id: FARM_ID })
        );
    }
}
