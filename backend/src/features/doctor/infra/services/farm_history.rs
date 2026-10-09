use std::sync::Arc;

use async_trait::async_trait;

use crate::{
    app::AppError as GlobalAppError,
    features::{
        doctor::{
            app::{AppError, FarmHistory},
            domain::{HistoryMeasure, HistoryTopic},
        },
        insights::{app::InsightRepository, domain::FarmInsight},
    },
};

/// Reads what is known about a farm by asking the insights feature through
/// its own repository port. The farm's ownership is checked before this is
/// called, so it is not checked again here.
#[derive(Debug)]
pub struct InsightsFeatureFarmHistory {
    insights: Arc<dyn InsightRepository>,
}

impl InsightsFeatureFarmHistory {
    pub fn new(insights: Arc<dyn InsightRepository>) -> Self {
        Self { insights }
    }
}

#[async_trait]
impl FarmHistory for InsightsFeatureFarmHistory {
    async fn history_of(&self, farm_id: i32) -> Result<Vec<HistoryTopic>, AppError> {
        let mut insights = self
            .insights
            .find_all_by_farm(farm_id)
            .await
            .map_err(|error| {
                tracing::error!(%error, farm_id, "reading the farm's history for the Doctor failed");

                GlobalAppError::InternalServerError
            })?;

        // The order of `GET /v1/farms/{id}/insights`, so the Doctor reads
        // the history as the farmer sees it.
        insights.sort_by_key(|insight| *insight.topic());

        Ok(insights.iter().map(topic_of).collect())
    }
}

fn topic_of(insight: &FarmInsight) -> HistoryTopic {
    HistoryTopic::rehydrate(
        String::from(*insight.topic()),
        *insight.as_of(),
        insight.source().into(),
        String::from(*insight.confidence()),
        insight.summary_en().as_ref().map(Into::into),
        insight.summary_ku().as_ref().map(Into::into),
        insight
            .measures()
            .iter()
            .map(|measure| {
                HistoryMeasure::rehydrate(
                    measure.code().into(),
                    *measure.value(),
                    measure.unit().clone(),
                    measure.label_en().clone(),
                    measure.label_ku().clone(),
                )
            })
            .collect(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::features::insights::{
        app::testing::{Call, FARM_ID, Fakes, an_insight},
        domain::Topic,
    };

    #[tokio::test]
    async fn the_history_is_every_topic_of_the_farm_in_the_apps_order() {
        let insights = Fakes::new()
            .with_stored(an_insight(FARM_ID, Topic::Weather, 8))
            .with_stored(an_insight(FARM_ID, Topic::SurfaceWater, 7))
            .with_stored(an_insight(99, Topic::Rain, 6));
        let history = InsightsFeatureFarmHistory::new(Arc::new(insights.clone()));

        let topics = history.history_of(FARM_ID).await.expect("history");

        assert_eq!(
            topics
                .iter()
                .map(|topic| topic.topic().as_str())
                .collect::<Vec<_>>(),
            vec!["surface_water", "weather"],
            "another farm's readings must never be mixed in"
        );
        assert_eq!(
            insights.calls(),
            vec![Call::FindAllByFarm { farm_id: FARM_ID }]
        );
    }

    #[tokio::test]
    async fn a_topic_carries_every_field_the_app_is_shown() {
        let insights = Fakes::new().with_stored(an_insight(FARM_ID, Topic::Rain, 6));
        let history = InsightsFeatureFarmHistory::new(Arc::new(insights));

        let topics = history.history_of(FARM_ID).await.expect("history");

        let topic = &topics[0];
        assert_eq!(topic.topic(), "rain");
        assert_eq!(topic.as_of().to_string(), "2026-10-06");
        assert_eq!(topic.source(), "Sentinel-2");
        assert_eq!(topic.confidence(), "likely");
        assert_eq!(topic.summary_en(), &None);
        assert_eq!(
            topic.measures(),
            &vec![HistoryMeasure::rehydrate(
                "level_pct".to_string(),
                54.2,
                "%".to_string(),
                "Dam level".to_string(),
                None,
            )]
        );
    }

    #[tokio::test]
    async fn a_farm_nothing_is_known_about_has_an_empty_history() {
        let history = InsightsFeatureFarmHistory::new(Arc::new(Fakes::new()));

        assert!(
            history
                .history_of(FARM_ID)
                .await
                .expect("history")
                .is_empty()
        );
    }

    #[tokio::test]
    async fn a_failure_in_the_insights_feature_is_an_error_not_an_empty_history() {
        let history = InsightsFeatureFarmHistory::new(Arc::new(Fakes::new().failing()));

        assert!(history.history_of(FARM_ID).await.is_err());
    }
}
