use std::sync::Arc;

use crate::features::outlooks::{
    app::{AppError, OutlookRepository},
    domain::{Confidence, IssueMonth, Outlook, Reason, Season, ZoneOutlook, ZoneSlug},
};

pub struct RecordZoneOutlookInput {
    pub zone_slug: ZoneSlug,
    pub season: Season,
    pub issued: IssueMonth,
    pub outlook: Outlook,
    pub confidence: Confidence,
    pub reason_en: Option<Reason>,
    pub reason_ku: Option<Reason>,
}

pub struct RecordZoneOutlookUseCase {
    repository: Arc<dyn OutlookRepository>,
}

impl RecordZoneOutlookUseCase {
    pub fn new(repository: Arc<dyn OutlookRepository>) -> Self {
        Self { repository }
    }

    /// Stores what the data job concluded for one zone in one issue. Sending
    /// the same zone, season and month again replaces the earlier outlook.
    pub async fn execute(&self, input: RecordZoneOutlookInput) -> Result<ZoneOutlook, AppError> {
        let outlook = ZoneOutlook::new(
            input.zone_slug,
            input.season,
            input.issued,
            input.outlook,
            input.confidence,
            input.reason_en,
            input.reason_ku,
        );

        let recorded = self.repository.upsert_zone_outlook(&outlook).await?;

        tracing::info!(
            zone = recorded.zone_slug().as_str(),
            season = recorded.season().as_str(),
            issued = %recorded.issued().first_day(),
            outlook = String::from(*recorded.outlook()),
            confidence_pct = recorded.confidence().value(),
            "zone outlook recorded"
        );

        Ok(recorded)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::features::outlooks::app::testing::{
        FakeOutlookRepository, RepositoryCall, month, season, zone_slug,
    };

    fn input() -> RecordZoneOutlookInput {
        RecordZoneOutlookInput {
            zone_slug: zone_slug("makhmur"),
            season: season("2026-27"),
            issued: month("2026-10"),
            outlook: Outlook::Bad,
            confidence: Confidence::new(80.0).expect("confidence"),
            reason_en: Some(Reason::new("Rain 40% below normal".to_string()).expect("reason")),
            reason_ku: Some(Reason::new("باران ٤٠٪ کەمترە".to_string()).expect("reason")),
        }
    }

    #[tokio::test]
    async fn records_the_outlook_under_its_zone_season_and_month() {
        let repository = FakeOutlookRepository::new();
        let use_case = RecordZoneOutlookUseCase::new(Arc::new(repository.clone()));

        let recorded = use_case.execute(input()).await.expect("outlook");

        assert!(recorded.id().is_some(), "the stored outlook comes back");
        assert_eq!(*recorded.outlook(), Outlook::Bad);
        assert_eq!(recorded.confidence().value(), 80.0);
        assert_eq!(
            recorded.reason_ku().as_ref().map(Reason::as_str),
            Some("باران ٤٠٪ کەمترە")
        );
        assert_eq!(
            repository.calls(),
            vec![RepositoryCall::UpsertZoneOutlook {
                zone_slug: "makhmur".to_string(),
                season: "2026-27".to_string(),
                issued: "2026-10".to_string(),
            }],
            "one write, keyed by the zone, the season and the month"
        );
    }

    #[tokio::test]
    async fn an_outlook_without_reasons_is_recorded_as_it_is() {
        let use_case = RecordZoneOutlookUseCase::new(Arc::new(FakeOutlookRepository::new()));

        let recorded = use_case
            .execute(RecordZoneOutlookInput {
                reason_en: None,
                reason_ku: None,
                ..input()
            })
            .await
            .expect("outlook");

        assert!(recorded.reason_en().is_none());
        assert!(recorded.reason_ku().is_none());
    }

    #[tokio::test]
    async fn a_repository_failure_surfaces() {
        let use_case = RecordZoneOutlookUseCase::new(Arc::new(FakeOutlookRepository::failing()));

        assert!(use_case.execute(input()).await.is_err());
    }
}
