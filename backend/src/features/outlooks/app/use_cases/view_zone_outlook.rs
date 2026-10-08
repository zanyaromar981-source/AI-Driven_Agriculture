use std::sync::Arc;

use crate::features::outlooks::{
    app::{AppError, OutlookRepository},
    domain::{Season, ZoneOutlook, ZoneSlug},
};

pub struct ViewZoneOutlookInput {
    pub zone_slug: ZoneSlug,
    /// None = the latest season that has an outlook.
    pub season: Option<Season>,
}

pub struct ViewZoneOutlookUseCase {
    repository: Arc<dyn OutlookRepository>,
}

impl ViewZoneOutlookUseCase {
    pub fn new(repository: Arc<dyn OutlookRepository>) -> Self {
        Self { repository }
    }

    /// Returns the season looked at and the zone's outlook at every issue of
    /// it, oldest first. A zone with no outlook in the season gets an empty
    /// list: whether the zone exists is for the zones slice to say.
    pub async fn execute(
        &self,
        input: ViewZoneOutlookInput,
    ) -> Result<(Season, Vec<ZoneOutlook>), AppError> {
        let season = match input.season {
            Some(season) => season,
            None => self
                .repository
                .find_latest_season()
                .await?
                .ok_or(AppError::NoOutlookIssued)?,
        };

        let history = self
            .repository
            .find_zone_history(&input.zone_slug, &season)
            .await?;

        tracing::debug!(
            zone = input.zone_slug.as_str(),
            season = season.as_str(),
            returned = history.len(),
            "zone outlook viewed"
        );

        Ok((season, history))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::features::outlooks::{
        app::testing::{
            FakeOutlookRepository, RepositoryCall, an_outlook, month, season, zone_slug,
        },
        domain::Outlook,
    };

    fn stored() -> FakeOutlookRepository {
        FakeOutlookRepository::holding(
            vec![
                an_outlook("makhmur", "2026-27", "2026-10", Outlook::Bad, 80.0),
                an_outlook("makhmur", "2026-27", "2026-08", Outlook::Normal, 50.0),
                an_outlook("makhmur", "2025-26", "2025-10", Outlook::Good, 70.0),
                an_outlook("koya", "2026-27", "2026-09", Outlook::Good, 60.0),
            ],
            vec![],
        )
    }

    fn input(zone: &str, for_season: Option<&str>) -> ViewZoneOutlookInput {
        ViewZoneOutlookInput {
            zone_slug: zone_slug(zone),
            season: for_season.map(season),
        }
    }

    #[tokio::test]
    async fn returns_the_zones_outlook_at_every_issue_of_the_latest_season_oldest_first() {
        let repository = stored();
        let use_case = ViewZoneOutlookUseCase::new(Arc::new(repository.clone()));

        let (season, history) = use_case
            .execute(input("makhmur", None))
            .await
            .expect("history");

        assert_eq!(season.as_str(), "2026-27");
        assert_eq!(
            history
                .iter()
                .map(|outlook| (*outlook.issued(), *outlook.outlook()))
                .collect::<Vec<_>>(),
            vec![
                (month("2026-08"), Outlook::Normal),
                (month("2026-10"), Outlook::Bad),
            ]
        );
        assert_eq!(
            repository.calls(),
            vec![
                RepositoryCall::FindLatestSeason,
                RepositoryCall::FindZoneHistory {
                    zone_slug: "makhmur".to_string(),
                    season: "2026-27".to_string(),
                },
            ]
        );
    }

    #[tokio::test]
    async fn a_named_season_is_used_as_given() {
        let repository = stored();
        let use_case = ViewZoneOutlookUseCase::new(Arc::new(repository.clone()));

        let (season, history) = use_case
            .execute(input("makhmur", Some("2025-26")))
            .await
            .expect("history");

        assert_eq!(season.as_str(), "2025-26");
        assert_eq!(history.len(), 1);
        assert_eq!(
            repository.calls(),
            vec![RepositoryCall::FindZoneHistory {
                zone_slug: "makhmur".to_string(),
                season: "2025-26".to_string(),
            }]
        );
    }

    #[tokio::test]
    async fn a_zone_without_an_outlook_in_the_season_gets_an_empty_list() {
        let use_case = ViewZoneOutlookUseCase::new(Arc::new(stored()));

        let (season, history) = use_case
            .execute(input("koya", Some("2025-26")))
            .await
            .expect("history");

        assert_eq!(season.as_str(), "2025-26");
        assert!(history.is_empty());
    }

    #[tokio::test]
    async fn with_nothing_stored_there_is_no_season_to_default_to() {
        let use_case = ViewZoneOutlookUseCase::new(Arc::new(FakeOutlookRepository::new()));

        let result = use_case.execute(input("makhmur", None)).await;

        assert!(matches!(result, Err(AppError::NoOutlookIssued)));
    }

    #[tokio::test]
    async fn a_repository_failure_surfaces() {
        let use_case = ViewZoneOutlookUseCase::new(Arc::new(FakeOutlookRepository::failing()));

        assert!(use_case.execute(input("makhmur", None)).await.is_err());
    }
}
