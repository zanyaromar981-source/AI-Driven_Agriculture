use std::sync::Arc;

use crate::{
    app::{AppError as GlobalAppError, StaffContext},
    features::insights::{
        app::{AppError, InsightRepository, use_cases::RecordFarmInsightInput},
        domain::FarmInsight,
    },
};

pub struct CorrectFarmInsightUseCase {
    repository: Arc<dyn InsightRepository>,
}

impl CorrectFarmInsightUseCase {
    pub fn new(repository: Arc<dyn InsightRepository>) -> Self {
        Self { repository }
    }

    /// Replaces the reading a farm has for a topic with what a staff member
    /// entered. Nothing is read first: whether the farm has such a reading
    /// is decided by the update itself. Unlike a push by a data job, a
    /// correction may describe an earlier day than the stored reading: the
    /// person is overriding it on purpose.
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

        let Some(stored) = self.repository.update(&insight).await? else {
            tracing::info!(
                staff_id = *actor.staff_id(),
                farm_id = *insight.farm_id(),
                topic = %String::from(*insight.topic()),
                "farm insight not corrected: the farm has no reading for this topic"
            );

            return Err(GlobalAppError::NotFound.into());
        };

        tracing::info!(
            staff_id = *actor.staff_id(),
            farm_id = *stored.farm_id(),
            topic = %String::from(*stored.topic()),
            as_of = %stored.as_of(),
            measures = stored.measures().len(),
            "farm insight corrected by staff"
        );

        Ok(stored)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::features::insights::{
        app::testing::{Call, FARM_ID, Fakes, a_day, a_measure, a_source, actor, an_insight},
        domain::{Confidence, InsightError, Topic},
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

    #[tokio::test]
    async fn replaces_the_stored_reading_in_one_call() {
        let fakes = Fakes::new().with_stored(an_insight(FARM_ID, Topic::Soil, 3));
        let use_case = CorrectFarmInsightUseCase::new(Arc::new(fakes.clone()));

        let stored = use_case
            .execute(&actor(), input(9))
            .await
            .expect("corrected");

        assert_eq!(*stored.as_of(), a_day(9));
        assert_eq!(stored.measures()[0].code().as_str(), "moisture_pct");
        assert_eq!(
            fakes.calls(),
            vec![Call::Update {
                farm_id: FARM_ID,
                topic: Topic::Soil,
            }],
            "the reading is not looked up before the write"
        );
    }

    #[tokio::test]
    async fn an_earlier_day_replaces_a_later_one() {
        let fakes = Fakes::new().with_stored(an_insight(FARM_ID, Topic::Soil, 9));
        let use_case = CorrectFarmInsightUseCase::new(Arc::new(fakes.clone()));

        let stored = use_case
            .execute(&actor(), input(2))
            .await
            .expect("corrected");

        assert_eq!(
            *stored.as_of(),
            a_day(2),
            "a staff correction is not held back by the newer stored reading"
        );
        assert_eq!(
            *fakes.stored(FARM_ID, Topic::Soil).expect("stored").as_of(),
            a_day(2)
        );
    }

    #[tokio::test]
    async fn a_topic_the_farm_has_no_reading_for_is_not_found_and_is_not_created() {
        let fakes = Fakes::new().with_stored(an_insight(FARM_ID, Topic::Rain, 3));
        let use_case = CorrectFarmInsightUseCase::new(Arc::new(fakes.clone()));

        let result = use_case.execute(&actor(), input(9)).await;

        assert!(matches!(
            result,
            Err(AppError::GlobalAppError(GlobalAppError::NotFound))
        ));
        assert!(fakes.stored(FARM_ID, Topic::Soil).is_none());
    }

    #[tokio::test]
    async fn a_correction_that_breaks_a_rule_is_not_written() {
        let fakes = Fakes::new().with_stored(an_insight(FARM_ID, Topic::Soil, 3));
        let use_case = CorrectFarmInsightUseCase::new(Arc::new(fakes.clone()));

        let result = use_case
            .execute(
                &actor(),
                RecordFarmInsightInput {
                    measures: vec![a_measure("depth_m"), a_measure("depth_m")],
                    ..input(9)
                },
            )
            .await;

        assert!(matches!(
            result,
            Err(AppError::Insight(InsightError::DuplicateMeasureCode(_)))
        ));
        assert!(fakes.calls().is_empty());
    }

    #[tokio::test]
    async fn a_repository_failure_surfaces() {
        let use_case = CorrectFarmInsightUseCase::new(Arc::new(Fakes::new().failing()));

        assert!(use_case.execute(&actor(), input(9)).await.is_err());
    }
}
