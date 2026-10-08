use std::sync::Arc;

use chrono::NaiveDate;

use crate::features::insights::{
    app::{AppError, InsightRepository},
    domain::{Confidence, FarmInsight, InsightSource, Measure, Summary, Topic},
};

pub struct RecordFarmInsightInput {
    pub farm_id: i32,
    pub topic: Topic,
    pub as_of: NaiveDate,
    pub source: InsightSource,
    pub confidence: Confidence,
    pub summary_en: Option<Summary>,
    pub summary_ku: Option<Summary>,
    pub measures: Vec<Measure>,
}

pub struct RecordFarmInsightUseCase {
    repository: Arc<dyn InsightRepository>,
}

impl RecordFarmInsightUseCase {
    pub fn new(repository: Arc<dyn InsightRepository>) -> Self {
        Self { repository }
    }

    /// Stores a data job's reading as the farm's current one for the topic.
    /// The farm is not looked up: the job takes its farm ids from the
    /// coverage list, and a reading for a farm deleted in between is never
    /// shown to anyone.
    pub async fn execute(&self, input: RecordFarmInsightInput) -> Result<FarmInsight, AppError> {
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

        let stored = self.repository.upsert(&insight).await?;

        tracing::info!(
            farm_id = *stored.farm_id(),
            topic = %String::from(*stored.topic()),
            as_of = %stored.as_of(),
            measures = stored.measures().len(),
            "farm insight recorded"
        );

        Ok(stored)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::features::insights::{
        app::testing::{Call, FARM_ID, Fakes, a_day, a_measure, a_source},
        domain::InsightError,
    };

    fn input(measures: Vec<Measure>) -> RecordFarmInsightInput {
        RecordFarmInsightInput {
            farm_id: FARM_ID,
            topic: Topic::Groundwater,
            as_of: a_day(7),
            source: a_source(),
            confidence: Confidence::Sure,
            summary_en: Some(Summary::new("The well level is steady.".to_string()).expect("text")),
            summary_ku: None,
            measures,
        }
    }

    #[tokio::test]
    async fn stores_the_reading_for_its_farm_and_topic() {
        let fakes = Fakes::new();
        let use_case = RecordFarmInsightUseCase::new(Arc::new(fakes.clone()));

        let stored = use_case
            .execute(input(vec![a_measure("depth_m")]))
            .await
            .expect("insight");

        assert!(stored.id().is_some(), "the answer is the stored reading");
        assert_eq!(*stored.as_of(), a_day(7));
        assert_eq!(
            fakes.calls(),
            vec![Call::Upsert {
                farm_id: FARM_ID,
                topic: Topic::Groundwater,
            }],
            "the farm is not looked up before the write"
        );
    }

    #[tokio::test]
    async fn the_measures_are_stored_as_they_were_pushed() {
        let use_case = RecordFarmInsightUseCase::new(Arc::new(Fakes::new()));

        let stored = use_case
            .execute(input(vec![a_measure("depth_m"), a_measure("trend_cm")]))
            .await
            .expect("insight");

        let codes: Vec<&str> = stored
            .measures()
            .iter()
            .map(|measure| measure.code().as_str())
            .collect();

        assert_eq!(codes, vec!["depth_m", "trend_cm"], "order is kept");
        assert_eq!(*stored.measures()[0].value(), 54.2);
    }

    #[tokio::test]
    async fn a_reading_without_measures_is_not_written() {
        let fakes = Fakes::new();
        let use_case = RecordFarmInsightUseCase::new(Arc::new(fakes.clone()));

        let result = use_case.execute(input(vec![])).await;

        assert!(matches!(
            result,
            Err(AppError::Insight(InsightError::MeasureCount { .. }))
        ));
        assert!(fakes.calls().is_empty());
    }

    #[tokio::test]
    async fn a_reading_with_a_repeated_code_is_not_written() {
        let fakes = Fakes::new();
        let use_case = RecordFarmInsightUseCase::new(Arc::new(fakes.clone()));

        let result = use_case
            .execute(input(vec![a_measure("depth_m"), a_measure("depth_m")]))
            .await;

        assert!(matches!(
            result,
            Err(AppError::Insight(InsightError::DuplicateMeasureCode(_)))
        ));
        assert!(fakes.calls().is_empty());
    }

    #[tokio::test]
    async fn a_repository_failure_surfaces() {
        let use_case = RecordFarmInsightUseCase::new(Arc::new(Fakes::new().failing()));

        assert!(
            use_case
                .execute(input(vec![a_measure("depth_m")]))
                .await
                .is_err()
        );
    }
}
