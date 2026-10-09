use std::sync::Arc;

use crate::{
    app::StaffContext,
    features::outlooks::{
        app::{AppError, OutlookRepository, use_cases::RecordZoneOutlookInput},
        domain::ZoneOutlook,
    },
};

pub struct UpdateZoneOutlookUseCase {
    repository: Arc<dyn OutlookRepository>,
}

impl UpdateZoneOutlookUseCase {
    pub fn new(repository: Arc<dyn OutlookRepository>) -> Self {
        Self { repository }
    }

    /// Replaces the stored outlook of one zone, season and issue with what
    /// a staff member corrected it to. Whether there is one is not looked up
    /// first: the update says how many rows it touched, and none means not
    /// found.
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

        let updated = self
            .repository
            .update_zone_outlook(&outlook)
            .await?
            .ok_or_else(|| AppError::ZoneOutlookNotFound {
                zone_slug: String::from(outlook.zone_slug()),
                season: String::from(outlook.season()),
                issued: String::from(outlook.issued()),
            })?;

        tracing::info!(
            staff_id = *actor.staff_id(),
            zone = updated.zone_slug().as_str(),
            season = updated.season().as_str(),
            issued = %updated.issued().first_day(),
            outlook = String::from(*updated.outlook()),
            confidence_pct = updated.confidence().value(),
            "zone outlook updated from the dashboard"
        );

        Ok(updated)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::features::outlooks::{
        app::testing::{
            FakeOutlookRepository, RepositoryCall, an_outlook, month, season, staff, zone_slug,
        },
        domain::{Confidence, Outlook},
    };

    fn input(zone: &str) -> RecordZoneOutlookInput {
        RecordZoneOutlookInput {
            zone_slug: zone_slug(zone),
            season: season("2026-27"),
            issued: month("2026-10"),
            outlook: Outlook::Bad,
            confidence: Confidence::new(72.5).expect("confidence"),
            reason_en: None,
            reason_ku: None,
        }
    }

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
    async fn replaces_the_outlook_of_that_zone_season_and_issue() {
        let repository = repository();
        let use_case = UpdateZoneOutlookUseCase::new(Arc::new(repository.clone()));

        let updated = use_case
            .execute(&staff(), input("makhmur"))
            .await
            .expect("outlook");

        assert_eq!(*updated.outlook(), Outlook::Bad);
        assert_eq!(updated.confidence().value(), 72.5);
        assert_eq!(
            repository.calls(),
            vec![RepositoryCall::UpdateZoneOutlook {
                zone_slug: "makhmur".to_string(),
                season: "2026-27".to_string(),
                issued: "2026-10".to_string(),
            }],
            "one update-only write, and no lookup before it"
        );
    }

    #[tokio::test]
    async fn a_key_without_an_outlook_is_not_found_and_none_is_created() {
        let repository = repository();
        let use_case = UpdateZoneOutlookUseCase::new(Arc::new(repository.clone()));

        let result = use_case.execute(&staff(), input("koya")).await;

        assert!(matches!(
            result,
            Err(AppError::ZoneOutlookNotFound { zone_slug, .. }) if zone_slug == "koya"
        ));
        assert!(
            !repository.calls().iter().any(|call| matches!(
                call,
                RepositoryCall::UpsertZoneOutlook { .. } | RepositoryCall::CreateZoneOutlook { .. }
            )),
            "an update must never fall back to creating"
        );
    }

    #[tokio::test]
    async fn a_repository_failure_surfaces() {
        let use_case = UpdateZoneOutlookUseCase::new(Arc::new(FakeOutlookRepository::failing()));

        assert!(use_case.execute(&staff(), input("makhmur")).await.is_err());
    }
}
