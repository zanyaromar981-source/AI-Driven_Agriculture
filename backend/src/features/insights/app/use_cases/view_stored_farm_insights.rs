use std::sync::Arc;

use crate::{
    app::AppError as GlobalAppError,
    features::insights::{
        app::{AppError, FarmDirectory, InsightRepository},
        domain::FarmInsight,
    },
};

pub struct ViewStoredFarmInsightsUseCase {
    repository: Arc<dyn InsightRepository>,
    directory: Arc<dyn FarmDirectory>,
}

impl ViewStoredFarmInsightsUseCase {
    pub fn new(repository: Arc<dyn InsightRepository>, directory: Arc<dyn FarmDirectory>) -> Self {
        Self {
            repository,
            directory,
        }
    }

    /// Returns the readings of any farmer's farm for the dashboard, in the
    /// order the topics are declared. A farm that does not exist is not
    /// found; one with no readings yet gives an empty list.
    pub async fn execute(&self, farm_id: i32) -> Result<Vec<FarmInsight>, AppError> {
        if !self.directory.exists(farm_id).await? {
            return Err(GlobalAppError::NotFound.into());
        }

        let mut insights = self.repository.find_all_by_farm(farm_id).await?;
        insights.sort_by_key(|insight| *insight.topic());

        tracing::debug!(
            farm_id,
            returned = insights.len(),
            "stored farm insights viewed"
        );

        Ok(insights)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::features::insights::{
        app::testing::{Call, FARM_ID, Fakes, an_insight},
        domain::Topic,
    };

    fn use_case(fakes: &Fakes) -> ViewStoredFarmInsightsUseCase {
        ViewStoredFarmInsightsUseCase::new(Arc::new(fakes.clone()), Arc::new(fakes.clone()))
    }

    #[tokio::test]
    async fn returns_the_readings_of_any_existing_farm_in_topic_order() {
        let fakes = Fakes::new()
            .with_site(99)
            .with_stored(an_insight(99, Topic::Weather, 8))
            .with_stored(an_insight(99, Topic::Groundwater, 7))
            .with_stored(an_insight(FARM_ID, Topic::Rain, 6));

        let insights = use_case(&fakes).execute(99).await.expect("insights");

        let topics: Vec<Topic> = insights.iter().map(|insight| *insight.topic()).collect();

        assert_eq!(topics, vec![Topic::Groundwater, Topic::Weather]);
        assert_eq!(
            fakes.calls(),
            vec![
                Call::Exists { farm_id: 99 },
                Call::FindAllByFarm { farm_id: 99 },
            ],
            "no owner is asked for: staff may read any farmer's farm"
        );
    }

    #[tokio::test]
    async fn a_farm_with_no_readings_yet_is_an_empty_list_not_an_error() {
        let fakes = Fakes::new().with_site(FARM_ID);

        assert!(
            use_case(&fakes)
                .execute(FARM_ID)
                .await
                .expect("insights")
                .is_empty()
        );
    }

    #[tokio::test]
    async fn a_farm_that_does_not_exist_is_not_found_even_with_readings_left_behind() {
        let fakes = Fakes::new().with_stored(an_insight(FARM_ID, Topic::Rain, 6));

        let result = use_case(&fakes).execute(FARM_ID).await;

        assert!(matches!(
            result,
            Err(AppError::GlobalAppError(GlobalAppError::NotFound))
        ));
        assert_eq!(fakes.calls(), vec![Call::Exists { farm_id: FARM_ID }]);
    }

    #[tokio::test]
    async fn a_failed_existence_check_surfaces_instead_of_reading() {
        let fakes = Fakes::new().with_site(FARM_ID).failing();

        assert!(use_case(&fakes).execute(FARM_ID).await.is_err());
        assert!(
            !fakes
                .calls()
                .contains(&Call::FindAllByFarm { farm_id: FARM_ID })
        );
    }
}
