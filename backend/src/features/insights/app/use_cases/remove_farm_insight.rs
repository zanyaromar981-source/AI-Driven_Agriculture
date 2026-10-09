use std::sync::Arc;

use crate::{
    app::StaffContext,
    features::insights::{
        app::{AppError, InsightRepository},
        domain::Topic,
    },
};

pub struct RemoveFarmInsightUseCase {
    repository: Arc<dyn InsightRepository>,
}

impl RemoveFarmInsightUseCase {
    pub fn new(repository: Arc<dyn InsightRepository>) -> Self {
        Self { repository }
    }

    /// Removes the reading a farm has for a topic. One that is already gone
    /// is a success, so a repeated delete gets the same answer as the first.
    /// The farm is not looked up, which also lets staff clear a reading
    /// left behind by a deleted farm.
    pub async fn execute(
        &self,
        actor: &StaffContext,
        farm_id: i32,
        topic: Topic,
    ) -> Result<(), AppError> {
        self.repository.delete(farm_id, topic).await?;

        tracing::info!(
            staff_id = *actor.staff_id(),
            farm_id,
            topic = %String::from(topic),
            "farm insight removed by staff"
        );

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::features::insights::app::testing::{Call, FARM_ID, Fakes, actor, an_insight};

    #[tokio::test]
    async fn removes_only_that_topic_of_that_farm_in_one_call() {
        let fakes = Fakes::new()
            .with_stored(an_insight(FARM_ID, Topic::Soil, 3))
            .with_stored(an_insight(FARM_ID, Topic::Rain, 3))
            .with_stored(an_insight(99, Topic::Soil, 3));
        let use_case = RemoveFarmInsightUseCase::new(Arc::new(fakes.clone()));

        use_case
            .execute(&actor(), FARM_ID, Topic::Soil)
            .await
            .expect("removed");

        assert!(fakes.stored(FARM_ID, Topic::Soil).is_none());
        assert!(fakes.stored(FARM_ID, Topic::Rain).is_some());
        assert!(fakes.stored(99, Topic::Soil).is_some());
        assert_eq!(
            fakes.calls(),
            vec![Call::Delete {
                farm_id: FARM_ID,
                topic: Topic::Soil,
            }]
        );
    }

    #[tokio::test]
    async fn removing_a_reading_that_is_already_gone_succeeds_again() {
        let use_case = RemoveFarmInsightUseCase::new(Arc::new(Fakes::new()));

        use_case
            .execute(&actor(), FARM_ID, Topic::Soil)
            .await
            .expect("first");
        use_case
            .execute(&actor(), FARM_ID, Topic::Soil)
            .await
            .expect("repeat");
    }

    #[tokio::test]
    async fn a_repository_failure_surfaces() {
        let use_case = RemoveFarmInsightUseCase::new(Arc::new(Fakes::new().failing()));

        assert!(
            use_case
                .execute(&actor(), FARM_ID, Topic::Soil)
                .await
                .is_err()
        );
    }
}
