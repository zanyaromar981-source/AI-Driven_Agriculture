use std::sync::Arc;

use crate::{
    app::{AppError as GlobalAppError, StaffContext},
    features::insights::{
        app::{AppError, FarmDirectory, InsightRepository, use_cases::RecordFarmInsightInput},
        domain::{FarmInsight, InsightError},
    },
};

pub struct CreateFarmInsightUseCase {
    repository: Arc<dyn InsightRepository>,
    directory: Arc<dyn FarmDirectory>,
}

impl CreateFarmInsightUseCase {
    pub fn new(repository: Arc<dyn InsightRepository>, directory: Arc<dyn FarmDirectory>) -> Self {
        Self {
            repository,
            directory,
        }
    }

    /// Stores a reading a staff member enters by hand for a topic the farm
    /// has none for. The farm is looked up only to answer "not found" for
    /// one that does not exist; whether the topic is free is decided by the
    /// insert itself, so of two creates sent at the same moment exactly one
    /// succeeds.
    pub async fn execute(
        &self,
        actor: &StaffContext,
        input: RecordFarmInsightInput,
    ) -> Result<FarmInsight, AppError> {
        let insight = FarmInsight::new(
            input.farm_id,
            input.topic,
            input.as_of,
            input.source,
            input.confidence,
            input.summary_en,
            input.summary_ku,
            input.measures,
        )?;

        if !self.directory.exists(*insight.farm_id()).await? {
            return Err(GlobalAppError::NotFound.into());
        }

        let Some(stored) = self.repository.create(&insight).await? else {
            tracing::info!(
                staff_id = *actor.staff_id(),
                farm_id = *insight.farm_id(),
                topic = %String::from(*insight.topic()),
                "farm insight not created: the farm already has this topic"
            );

            return Err(InsightError::AlreadyExists.into());
        };

        tracing::info!(
            staff_id = *actor.staff_id(),
            farm_id = *stored.farm_id(),
            topic = %String::from(*stored.topic()),
            as_of = %stored.as_of(),
            measures = stored.measures().len(),
            "farm insight created by staff"
        );

        Ok(stored)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::features::insights::{
        app::testing::{Call, FARM_ID, Fakes, a_day, a_measure, a_source, actor, an_insight},
        domain::{Confidence, Topic},
    };

    fn input(day: u32) -> RecordFarmInsightInput {
        RecordFarmInsightInput {
            farm_id: FARM_ID,
            topic: Topic::Soil,
            as_of: a_day(day),
            source: a_source(),
            confidence: Confidence::Sure,
            summary_en: None,
            summary_ku: None,
            measures: vec![a_measure("moisture_pct")],
        }
    }

    fn use_case(fakes: &Fakes) -> CreateFarmInsightUseCase {
        CreateFarmInsightUseCase::new(Arc::new(fakes.clone()), Arc::new(fakes.clone()))
    }

    #[tokio::test]
    async fn stores_a_new_reading_for_an_existing_farm() {
        let fakes = Fakes::new().with_site(FARM_ID);

        let stored = use_case(&fakes)
            .execute(&actor(), input(7))
            .await
            .expect("insight");

        assert!(stored.id().is_some(), "the answer is the stored reading");
        assert_eq!(
            fakes.calls(),
            vec![
                Call::Exists { farm_id: FARM_ID },
                Call::Create {
                    farm_id: FARM_ID,
                    topic: Topic::Soil,
                },
            ],
            "the stored readings are not looked up before the write"
        );
    }

    #[tokio::test]
    async fn a_topic_the_farm_already_has_is_refused_and_nothing_changes() {
        let fakes =
            Fakes::new()
                .with_site(FARM_ID)
                .with_stored(an_insight(FARM_ID, Topic::Soil, 3));

        let result = use_case(&fakes).execute(&actor(), input(9)).await;

        assert!(matches!(
            result,
            Err(AppError::Insight(InsightError::AlreadyExists))
        ));
        assert_eq!(
            *fakes.stored(FARM_ID, Topic::Soil).expect("stored").as_of(),
            a_day(3),
            "a create must never replace the stored reading"
        );
    }

    #[tokio::test]
    async fn the_same_topic_of_another_farm_does_not_stand_in_the_way() {
        let fakes = Fakes::new()
            .with_site(FARM_ID)
            .with_stored(an_insight(99, Topic::Soil, 3));

        assert!(use_case(&fakes).execute(&actor(), input(7)).await.is_ok());
    }

    #[tokio::test]
    async fn a_farm_that_does_not_exist_is_not_found_and_nothing_is_written() {
        let fakes = Fakes::new();

        let result = use_case(&fakes).execute(&actor(), input(7)).await;

        assert!(matches!(
            result,
            Err(AppError::GlobalAppError(GlobalAppError::NotFound))
        ));
        assert_eq!(fakes.calls(), vec![Call::Exists { farm_id: FARM_ID }]);
    }

    #[tokio::test]
    async fn a_reading_that_breaks_a_rule_is_refused_before_anything_is_asked() {
        let fakes = Fakes::new().with_site(FARM_ID);

        let result = use_case(&fakes)
            .execute(
                &actor(),
                RecordFarmInsightInput {
                    measures: vec![],
                    ..input(7)
                },
            )
            .await;

        assert!(matches!(
            result,
            Err(AppError::Insight(InsightError::MeasureCount { .. }))
        ));
        assert!(fakes.calls().is_empty());
    }

    #[tokio::test]
    async fn a_repository_failure_surfaces() {
        let fakes = Fakes::new().with_site(FARM_ID).failing();

        assert!(use_case(&fakes).execute(&actor(), input(7)).await.is_err());
    }
}
