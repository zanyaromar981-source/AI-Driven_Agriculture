use std::sync::Arc;

use crate::{
    app::StaffContext,
    features::outlooks::{
        app::{AppError, OutlookRepository},
        domain::{IssueMonth, Season, ZoneSlug},
    },
};

pub struct DeleteZoneOutlookUseCase {
    repository: Arc<dyn OutlookRepository>,
}

impl DeleteZoneOutlookUseCase {
    pub fn new(repository: Arc<dyn OutlookRepository>) -> Self {
        Self { repository }
    }

    /// Removes one zone's outlook for one season and issue. Removing one
    /// that is not there succeeds: a repeat of a call whose answer was lost
    /// asked for the outlook to be gone, and it is.
    pub async fn execute(
        &self,
        actor: &StaffContext,
        zone_slug: ZoneSlug,
        season: Season,
        issued: IssueMonth,
    ) -> Result<(), AppError> {
        let removed = self
            .repository
            .delete_zone_outlook(&zone_slug, &season, &issued)
            .await?;

        tracing::info!(
            staff_id = *actor.staff_id(),
            zone = zone_slug.as_str(),
            season = season.as_str(),
            issued = %issued.first_day(),
            removed,
            "zone outlook deleted from the dashboard, or already gone"
        );

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::features::outlooks::{
        app::testing::{
            FakeOutlookRepository, RepositoryCall, an_outlook, month, season, staff, zone_slug,
        },
        domain::Outlook,
    };

    fn repository() -> FakeOutlookRepository {
        FakeOutlookRepository::holding(
            vec![an_outlook(
                "makhmur",
                "2026-27",
                "2026-10",
                Outlook::Good,
                50.0,
            )],
            vec![],
        )
    }

    #[tokio::test]
    async fn removes_the_outlook_named_by_the_whole_key() {
        let repository = repository();
        let use_case = DeleteZoneOutlookUseCase::new(Arc::new(repository.clone()));

        use_case
            .execute(
                &staff(),
                zone_slug("makhmur"),
                season("2026-27"),
                month("2026-10"),
            )
            .await
            .expect("deleted");

        assert_eq!(
            repository.calls(),
            vec![RepositoryCall::DeleteZoneOutlook {
                zone_slug: "makhmur".to_string(),
                season: "2026-27".to_string(),
                issued: "2026-10".to_string(),
            }],
            "the season and the issue must be part of the delete, never the zone alone"
        );
    }

    #[tokio::test]
    async fn deleting_an_outlook_that_is_already_gone_succeeds() {
        let use_case = DeleteZoneOutlookUseCase::new(Arc::new(repository()));

        let actor = staff();
        let delete = || {
            use_case.execute(
                &actor,
                zone_slug("makhmur"),
                season("2026-27"),
                month("2026-10"),
            )
        };

        assert!(delete().await.is_ok());
        assert!(delete().await.is_ok(), "a repeated delete must not fail");
    }

    #[tokio::test]
    async fn a_repository_failure_surfaces() {
        let use_case = DeleteZoneOutlookUseCase::new(Arc::new(FakeOutlookRepository::failing()));

        let result = use_case
            .execute(
                &staff(),
                zone_slug("makhmur"),
                season("2026-27"),
                month("2026-10"),
            )
            .await;

        assert!(result.is_err());
    }
}
