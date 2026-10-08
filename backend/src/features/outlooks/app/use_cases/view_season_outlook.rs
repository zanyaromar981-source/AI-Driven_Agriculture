use std::sync::Arc;

use crate::features::outlooks::{
    app::{AppError, OutlookRepository},
    domain::{IssueMonth, Season, SeasonOutlook},
};

pub struct ViewSeasonOutlookInput {
    /// None = the latest season that has an outlook.
    pub season: Option<Season>,
    /// None = the latest issue of the season.
    pub issued: Option<IssueMonth>,
}

pub struct ViewSeasonOutlookUseCase {
    repository: Arc<dyn OutlookRepository>,
}

impl ViewSeasonOutlookUseCase {
    pub fn new(repository: Arc<dyn OutlookRepository>) -> Self {
        Self { repository }
    }

    /// Returns one issue of the outlook for every zone. A season or a month
    /// nothing was issued for is not found: there is no empty outlook to
    /// show in its place.
    pub async fn execute(&self, input: ViewSeasonOutlookInput) -> Result<SeasonOutlook, AppError> {
        let season = match input.season {
            Some(season) => season,
            None => self
                .repository
                .find_latest_season()
                .await?
                .ok_or(AppError::NoOutlookIssued)?,
        };

        let issues = self.repository.find_issue_months(&season).await?;

        let Some(latest) = issues.iter().max().copied() else {
            return Err(AppError::SeasonNotIssued(String::from(&season)));
        };

        let issued = match input.issued {
            Some(issued) if issues.contains(&issued) => issued,
            Some(issued) => {
                return Err(AppError::IssueNotFound {
                    season: String::from(&season),
                    issued: String::from(&issued),
                });
            }
            None => latest,
        };

        let zones = self.repository.find_zone_outlooks(&season, &issued).await?;
        let track_record = self.repository.find_run(&season, &issued).await?;

        tracing::debug!(
            season = season.as_str(),
            issued = %issued.first_day(),
            zones = zones.len(),
            has_track_record = track_record.is_some(),
            "season outlook viewed"
        );

        Ok(SeasonOutlook::new(
            season,
            issued,
            zones,
            track_record,
            issues,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::features::outlooks::{
        app::testing::{FakeOutlookRepository, RepositoryCall, a_run, an_outlook, month, season},
        domain::{Outlook, OutlookCounts},
    };

    fn stored() -> FakeOutlookRepository {
        FakeOutlookRepository::holding(
            vec![
                an_outlook("makhmur", "2025-26", "2025-10", Outlook::Good, 70.0),
                an_outlook("makhmur", "2026-27", "2026-09", Outlook::Normal, 55.0),
                an_outlook("makhmur", "2026-27", "2026-10", Outlook::Bad, 80.0),
                an_outlook("koya", "2026-27", "2026-10", Outlook::Good, 60.0),
                an_outlook("kalar", "2026-27", "2026-10", Outlook::Bad, 90.0),
            ],
            vec![a_run("2026-27", "2026-10")],
        )
    }

    fn input(for_season: Option<&str>, issued: Option<&str>) -> ViewSeasonOutlookInput {
        ViewSeasonOutlookInput {
            season: for_season.map(season),
            issued: issued.map(month),
        }
    }

    fn slugs(board: &SeasonOutlook) -> Vec<&str> {
        board
            .zones()
            .iter()
            .map(|zone| zone.zone_slug().as_str())
            .collect()
    }

    #[tokio::test]
    async fn without_a_season_or_a_month_the_latest_of_both_is_shown() {
        let repository = stored();
        let use_case = ViewSeasonOutlookUseCase::new(Arc::new(repository.clone()));

        let board = use_case.execute(input(None, None)).await.expect("board");

        assert_eq!(board.season().as_str(), "2026-27");
        assert_eq!(*board.issued(), month("2026-10"));
        assert_eq!(slugs(&board), vec!["kalar", "makhmur", "koya"]);
        assert_eq!(
            *board.counts(),
            OutlookCounts {
                good: 1,
                normal: 0,
                bad: 2
            }
        );
        assert_eq!(*board.issues(), vec![month("2026-09"), month("2026-10")]);
        assert_eq!(
            repository.calls(),
            vec![
                RepositoryCall::FindLatestSeason,
                RepositoryCall::FindIssueMonths {
                    season: "2026-27".to_string()
                },
                RepositoryCall::FindZoneOutlooks {
                    season: "2026-27".to_string(),
                    issued: "2026-10".to_string(),
                },
                RepositoryCall::FindRun {
                    season: "2026-27".to_string(),
                    issued: "2026-10".to_string(),
                },
            ]
        );
    }

    #[tokio::test]
    async fn a_named_season_is_shown_without_looking_for_the_latest() {
        let repository = stored();
        let use_case = ViewSeasonOutlookUseCase::new(Arc::new(repository.clone()));

        let board = use_case
            .execute(input(Some("2025-26"), None))
            .await
            .expect("board");

        assert_eq!(board.season().as_str(), "2025-26");
        assert_eq!(*board.issued(), month("2025-10"));
        assert!(
            !repository
                .calls()
                .contains(&RepositoryCall::FindLatestSeason)
        );
    }

    #[tokio::test]
    async fn an_earlier_issue_can_be_asked_for_and_still_lists_every_issue() {
        let use_case = ViewSeasonOutlookUseCase::new(Arc::new(stored()));

        let board = use_case
            .execute(input(Some("2026-27"), Some("2026-09")))
            .await
            .expect("board");

        assert_eq!(*board.issued(), month("2026-09"));
        assert_eq!(slugs(&board), vec!["makhmur"]);
        assert_eq!(*board.issues(), vec![month("2026-09"), month("2026-10")]);
    }

    #[tokio::test]
    async fn the_track_record_is_that_of_the_issue_shown_or_missing() {
        let use_case = ViewSeasonOutlookUseCase::new(Arc::new(stored()));

        let latest = use_case.execute(input(None, None)).await.expect("board");
        let earlier = use_case
            .execute(input(None, Some("2026-09")))
            .await
            .expect("board");

        assert!(latest.track_record().is_some());
        assert!(
            earlier.track_record().is_none(),
            "the record of another issue must not be shown under this one"
        );
    }

    #[tokio::test]
    async fn with_nothing_stored_there_is_no_outlook_to_show() {
        let repository = FakeOutlookRepository::new();
        let use_case = ViewSeasonOutlookUseCase::new(Arc::new(repository.clone()));

        let result = use_case.execute(input(None, None)).await;

        assert!(matches!(result, Err(AppError::NoOutlookIssued)));
        assert_eq!(repository.calls(), vec![RepositoryCall::FindLatestSeason]);
    }

    #[tokio::test]
    async fn a_season_nothing_was_issued_for_is_not_found() {
        let use_case = ViewSeasonOutlookUseCase::new(Arc::new(stored()));

        let result = use_case.execute(input(Some("2030-31"), None)).await;

        assert!(matches!(result, Err(AppError::SeasonNotIssued(season)) if season == "2030-31"));
    }

    #[tokio::test]
    async fn a_month_nothing_was_issued_in_is_not_found() {
        let repository = stored();
        let use_case = ViewSeasonOutlookUseCase::new(Arc::new(repository.clone()));

        let result = use_case
            .execute(input(Some("2026-27"), Some("2026-07")))
            .await;

        assert!(matches!(
            result,
            Err(AppError::IssueNotFound { season, issued })
                if season == "2026-27" && issued == "2026-07"
        ));
        assert!(
            !repository
                .calls()
                .iter()
                .any(|call| matches!(call, RepositoryCall::FindZoneOutlooks { .. })),
            "the zones are not read for an issue that does not exist"
        );
    }

    #[tokio::test]
    async fn a_repository_failure_surfaces() {
        let use_case = ViewSeasonOutlookUseCase::new(Arc::new(FakeOutlookRepository::failing()));

        assert!(use_case.execute(input(None, None)).await.is_err());
    }
}
