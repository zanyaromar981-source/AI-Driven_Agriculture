use std::sync::Arc;

use chrono::{DateTime, NaiveDate, Utc};

use crate::features::briefs::{
    app::{AppError, BriefRepository},
    domain::{Author, BriefPoint, BriefScope, BriefSource, BriefSummary, DailyBrief, Headline},
};

pub struct RecordBriefInput {
    pub day: NaiveDate,
    pub scope: BriefScope,
    pub headline_en: Headline,
    pub headline_ku: Headline,
    pub summary_en: BriefSummary,
    pub summary_ku: BriefSummary,
    pub points: Vec<BriefPoint>,
    pub sources: Vec<BriefSource>,
    pub author: Author,
    pub generated_at: DateTime<Utc>,
}

impl RecordBriefInput {
    pub fn into_brief(self) -> Result<DailyBrief, AppError> {
        Ok(DailyBrief::new(
            self.day,
            self.scope,
            self.headline_en,
            self.headline_ku,
            self.summary_en,
            self.summary_ku,
            self.points,
            self.sources,
            self.author,
            self.generated_at,
        )?)
    }
}

pub struct RecordBriefUseCase {
    repository: Arc<dyn BriefRepository>,
}

impl RecordBriefUseCase {
    pub fn new(repository: Arc<dyn BriefRepository>) -> Self {
        Self { repository }
    }

    /// Stores the nightly job's brief as the one for its day and scope. A
    /// repeat of the push, or a second run of the job for the same day,
    /// replaces the brief as a whole. The scope is not looked up among the
    /// districts: a brief for a slug no farm lies in is never shown to a
    /// farmer.
    pub async fn execute(&self, input: RecordBriefInput) -> Result<DailyBrief, AppError> {
        let brief = input.into_brief()?;

        let stored = self.repository.upsert(&brief).await?;

        tracing::info!(
            day = %stored.day(),
            scope = stored.scope().as_str(),
            author = stored.author().as_str(),
            points = stored.points().len(),
            sources = stored.sources().len(),
            "brief recorded"
        );

        Ok(stored)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::features::briefs::{
        app::testing::{Call, Fakes, a_brief, a_day, a_point, an_input},
        domain::{BriefError, MAX_POINTS},
    };

    #[tokio::test]
    async fn stores_the_brief_for_its_day_and_scope_in_one_call() {
        let fakes = Fakes::new();
        let use_case = RecordBriefUseCase::new(Arc::new(fakes.clone()));

        let stored = use_case
            .execute(an_input(9, "chamchamal", "Rain at last"))
            .await
            .expect("brief");

        assert!(stored.id().is_some(), "the answer is the stored brief");
        assert_eq!(stored.headline_en().as_str(), "Rain at last");
        assert_eq!(
            fakes.calls(),
            vec![Call::Upsert {
                day: a_day(9),
                scope: "chamchamal".to_string(),
            }],
            "nothing is looked up before the write"
        );
    }

    #[tokio::test]
    async fn a_second_push_for_the_same_day_and_scope_replaces_the_first() {
        let fakes = Fakes::new()
            .with_stored(a_brief(9, "region"))
            .with_stored(a_brief(8, "region"));
        let use_case = RecordBriefUseCase::new(Arc::new(fakes.clone()));

        use_case
            .execute(an_input(9, "region", "Corrected headline"))
            .await
            .expect("brief");

        assert_eq!(
            fakes
                .stored(a_day(9), "region")
                .expect("stored")
                .headline_en()
                .as_str(),
            "Corrected headline"
        );
        assert_eq!(
            fakes
                .stored(a_day(8), "region")
                .expect("stored")
                .headline_en()
                .as_str(),
            "A dry week",
            "another day's brief is left alone"
        );
    }

    #[tokio::test]
    async fn the_points_are_stored_in_the_order_they_were_pushed() {
        let use_case = RecordBriefUseCase::new(Arc::new(Fakes::new()));

        let stored = use_case
            .execute(RecordBriefInput {
                points: vec![a_point("first"), a_point("second")],
                ..an_input(9, "region", "A dry week")
            })
            .await
            .expect("brief");

        let texts: Vec<&str> = stored
            .points()
            .iter()
            .map(|point| point.text_en().as_str())
            .collect();

        assert_eq!(texts, vec!["first", "second"]);
    }

    #[tokio::test]
    async fn a_brief_with_too_many_points_is_not_written() {
        let fakes = Fakes::new();
        let use_case = RecordBriefUseCase::new(Arc::new(fakes.clone()));

        let result = use_case
            .execute(RecordBriefInput {
                points: vec![a_point("one"); MAX_POINTS + 1],
                ..an_input(9, "region", "A dry week")
            })
            .await;

        assert!(matches!(
            result,
            Err(AppError::Brief(BriefError::TooManyPoints { .. }))
        ));
        assert!(fakes.calls().is_empty());
    }

    #[tokio::test]
    async fn a_repository_failure_surfaces() {
        let use_case = RecordBriefUseCase::new(Arc::new(Fakes::new().failing()));

        assert!(
            use_case
                .execute(an_input(9, "region", "A dry week"))
                .await
                .is_err()
        );
    }
}
