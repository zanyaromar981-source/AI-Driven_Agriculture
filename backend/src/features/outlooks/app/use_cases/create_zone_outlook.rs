use std::sync::Arc;

use crate::{
    app::StaffContext,
    features::outlooks::{
        app::{AppError, OutlookRepository, use_cases::RecordZoneOutlookInput},
        domain::ZoneOutlook,
    },
};

pub struct CreateZoneOutlookUseCase {
    repository: Arc<dyn OutlookRepository>,
}

impl CreateZoneOutlookUseCase {
    pub fn new(repository: Arc<dyn OutlookRepository>) -> Self {
        Self { repository }
    }

    /// Stores an outlook a staff member typed in for a zone, season and
    /// issue that has none. Whether the key is free is not looked up first:
    /// the unique index answers that when the outlook is stored, so of two
    /// copies sent at the same moment exactly one is created.
    pub async fn execute(
        &self,
        actor: &StaffContext,
        input: RecordZoneOutlookInput,
    ) -> Result<ZoneOutlook, AppError> {
        let outlook = ZoneOutlook::new(
            input.zone_slug,
            input.season,
            input.issued,
            input.outlook,
            input.confidence,
            input.reason_en,
            input.reason_ku,
        );

        let created = self
            .repository
            .create_zone_outlook(&outlook)
            .await?
            .ok_or_else(|| AppError::ZoneOutlookAlreadyExists {
                zone_slug: String::from(outlook.zone_slug()),
                season: String::from(outlook.season()),
                issued: String::from(outlook.issued()),
            })?;

        tracing::info!(
            staff_id = *actor.staff_id(),
            zone = created.zone_slug().as_str(),
            season = created.season().as_str(),
            issued = %created.issued().first_day(),
            outlook = String::from(*created.outlook()),
            confidence_pct = created.confidence().value(),
            "zone outlook created from the dashboard"
        );

        Ok(created)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::features::outlooks::{
        app::testing::{
            FakeOutlookRepository, RepositoryCall, an_outlook, month, season, staff, zone_slug,
        },
        domain::{Confidence, Outlook, Reason},
    };

    fn input(zone: &str) -> RecordZoneOutlookInput {
        RecordZoneOutlookInput {
            zone_slug: zone_slug(zone),
            season: season("2026-27"),
            issued: month("2026-10"),
            outlook: Outlook::Bad,
            confidence: Confidence::new(72.5).expect("confidence"),
            reason_en: Some(Reason::new("Dry autumn".to_string()).expect("reason")),
            reason_ku: None,
        }
    }

    #[tokio::test]
    async fn creates_the_outlook_under_its_zone_season_and_issue() {
        let repository = FakeOutlookRepository::new();
        let use_case = CreateZoneOutlookUseCase::new(Arc::new(repository.clone()));

        let created = use_case
            .execute(&staff(), input("makhmur"))
            .await
            .expect("outlook");

        assert!(created.id().is_some(), "the stored outlook comes back");
        assert_eq!(*created.outlook(), Outlook::Bad);
        assert_eq!(created.confidence().value(), 72.5);
        assert_eq!(
            repository.calls(),
            vec![RepositoryCall::CreateZoneOutlook {
                zone_slug: "makhmur".to_string(),
                season: "2026-27".to_string(),
                issued: "2026-10".to_string(),
            }],
            "one create-only write, and no lookup before it"
        );
    }

    #[tokio::test]
    async fn a_key_that_already_has_an_outlook_is_refused_and_left_as_it_was() {
        let repository = FakeOutlookRepository::holding(
            vec![an_outlook(
                "makhmur",
                "2026-27",
                "2026-10",
                Outlook::Good,
                50.0,
            )],
            vec![],
        );
        let use_case = CreateZoneOutlookUseCase::new(Arc::new(repository.clone()));

        let result = use_case.execute(&staff(), input("makhmur")).await;

        assert!(matches!(
            result,
            Err(AppError::ZoneOutlookAlreadyExists { zone_slug, season, issued })
                if zone_slug == "makhmur" && season == "2026-27" && issued == "2026-10"
        ));
        assert!(
            !repository
                .calls()
                .iter()
                .any(|call| matches!(call, RepositoryCall::UpsertZoneOutlook { .. })),
            "a create must never fall back to replacing"
        );
    }

    #[tokio::test]
    async fn another_zone_of_the_same_issue_is_free() {
        let repository = FakeOutlookRepository::holding(
            vec![an_outlook(
                "makhmur",
                "2026-27",
                "2026-10",
                Outlook::Good,
                50.0,
            )],
            vec![],
        );
        let use_case = CreateZoneOutlookUseCase::new(Arc::new(repository));

        assert!(use_case.execute(&staff(), input("koya")).await.is_ok());
    }

    #[tokio::test]
    async fn a_repository_failure_surfaces() {
        let use_case = CreateZoneOutlookUseCase::new(Arc::new(FakeOutlookRepository::failing()));

        assert!(use_case.execute(&staff(), input("makhmur")).await.is_err());
    }
}
