use std::sync::Arc;

use crate::{
    app::Pagination,
    features::outlooks::{
        app::{AppError, OutlookRepository},
        domain::{IssueMonth, Season, ZoneOutlook},
    },
};

pub struct ListZoneOutlooksInput {
    /// None = every season.
    pub season: Option<Season>,
    /// None = every issue.
    pub issued: Option<IssueMonth>,
    pub pagination: Pagination,
}

pub struct ListZoneOutlooksUseCase {
    repository: Arc<dyn OutlookRepository>,
}

impl ListZoneOutlooksUseCase {
    pub fn new(repository: Arc<dyn OutlookRepository>) -> Self {
        Self { repository }
    }

    /// Returns one page of the stored zone outlooks as they are, newest
    /// issue first, and how many match in all. Unlike the public board,
    /// nothing is defaulted to the latest season and nothing is ranked.
    pub async fn execute(
        &self,
        input: ListZoneOutlooksInput,
    ) -> Result<(Vec<ZoneOutlook>, u64), AppError> {
        let (outlooks, count) = self
            .repository
            .find_zone_outlooks_page(
                input.season.as_ref(),
                input.issued.as_ref(),
                &input.pagination,
            )
            .await?;

        tracing::debug!(returned = outlooks.len(), count, "zone outlooks listed");

        Ok((outlooks, count))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::features::outlooks::{
        app::testing::{FakeOutlookRepository, RepositoryCall, an_outlook, month, season},
        domain::Outlook,
    };

    fn repository() -> FakeOutlookRepository {
        FakeOutlookRepository::holding(
            vec![
                an_outlook("koya", "2026-27", "2026-09", Outlook::Good, 60.0),
                an_outlook("makhmur", "2026-27", "2026-10", Outlook::Bad, 80.0),
                an_outlook("koya", "2026-27", "2026-10", Outlook::Normal, 55.0),
                an_outlook("koya", "2025-26", "2025-10", Outlook::Bad, 70.0),
            ],
            vec![],
        )
    }

    fn keys(outlooks: &[ZoneOutlook]) -> Vec<(String, String)> {
        outlooks
            .iter()
            .map(|outlook| {
                (
                    String::from(outlook.issued()),
                    String::from(outlook.zone_slug()),
                )
            })
            .collect()
    }

    #[tokio::test]
    async fn without_filters_every_outlook_is_listed_newest_issue_first() {
        let repository = repository();
        let use_case = ListZoneOutlooksUseCase::new(Arc::new(repository.clone()));

        let (outlooks, count) = use_case
            .execute(ListZoneOutlooksInput {
                season: None,
                issued: None,
                pagination: Pagination::new(1, 3),
            })
            .await
            .expect("outlooks");

        assert_eq!(
            keys(&outlooks),
            vec![
                ("2026-10".to_string(), "koya".to_string()),
                ("2026-10".to_string(), "makhmur".to_string()),
                ("2026-09".to_string(), "koya".to_string()),
            ]
        );
        assert_eq!(count, 4, "the count covers every page");
        assert_eq!(
            repository.calls(),
            vec![RepositoryCall::FindZoneOutlooksPage {
                season: None,
                issued: None,
                page: 1,
                rows_per_page: 3,
            }],
            "no latest season is looked up: an editing screen sees all of them"
        );
    }

    #[tokio::test]
    async fn the_season_and_the_issue_narrow_the_list() {
        let repository = repository();
        let use_case = ListZoneOutlooksUseCase::new(Arc::new(repository.clone()));

        let (outlooks, count) = use_case
            .execute(ListZoneOutlooksInput {
                season: Some(season("2026-27")),
                issued: Some(month("2026-09")),
                pagination: Pagination::new(1, 100),
            })
            .await
            .expect("outlooks");

        assert_eq!(
            keys(&outlooks),
            vec![("2026-09".to_string(), "koya".to_string())]
        );
        assert_eq!(count, 1);
        assert_eq!(
            repository.calls(),
            vec![RepositoryCall::FindZoneOutlooksPage {
                season: Some("2026-27".to_string()),
                issued: Some("2026-09".to_string()),
                page: 1,
                rows_per_page: 100,
            }]
        );
    }

    #[tokio::test]
    async fn a_repository_failure_surfaces() {
        let use_case = ListZoneOutlooksUseCase::new(Arc::new(FakeOutlookRepository::failing()));

        let result = use_case
            .execute(ListZoneOutlooksInput {
                season: None,
                issued: None,
                pagination: Pagination::new(1, 100),
            })
            .await;

        assert!(result.is_err());
    }
}
