use std::sync::Arc;

use crate::{
    app::StaffContext,
    features::water::{
        app::{AppError, WaterPlanRepository, use_cases::SetWaterPlanEntryInput},
        domain::{DamSlug, WaterPlanEntry},
    },
};

pub struct UpdateWaterPlanEntryUseCase {
    repository: Arc<dyn WaterPlanRepository>,
}

impl UpdateWaterPlanEntryUseCase {
    pub fn new(repository: Arc<dyn WaterPlanRepository>) -> Self {
        Self { repository }
    }

    /// Replaces one zone's line in a season's plan as a whole. Whether the
    /// line is there is not looked up first: the update says how many rows
    /// it touched, and none means not found.
    pub async fn execute(
        &self,
        actor: &StaffContext,
        input: SetWaterPlanEntryInput,
    ) -> Result<WaterPlanEntry, AppError> {
        let entry = WaterPlanEntry::new(
            input.season,
            input.zone_slug,
            input.need,
            input.dam_slug,
            input.send_million_m3,
            input.urgent,
            input.note_en,
            input.note_ku,
        )?;

        let updated =
            self.repository
                .update(&entry)
                .await?
                .ok_or_else(|| AppError::EntryNotFound {
                    season: String::from(entry.season()),
                    zone_slug: String::from(entry.zone_slug()),
                })?;

        tracing::info!(
            staff_id = *actor.staff_id(),
            season = updated.season().as_str(),
            zone = updated.zone_slug().as_str(),
            need = updated.need().value(),
            dam = updated.dam_slug().as_ref().map(DamSlug::as_str),
            send_million_m3 = *updated.send_million_m3(),
            urgent = *updated.urgent(),
            "water plan entry updated from the dashboard"
        );

        Ok(updated)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::features::water::{
        app::testing::{
            FakeWaterPlanRepository, RepositoryCall, an_entry, season, staff, zone_slug,
        },
        domain::Need,
    };

    fn input(zone: &str, send_million_m3: Option<f64>) -> SetWaterPlanEntryInput {
        SetWaterPlanEntryInput {
            season: season("2026-27"),
            zone_slug: zone_slug(zone),
            need: Need::new(75.0).expect("need"),
            dam_slug: None,
            send_million_m3,
            urgent: false,
            note_en: None,
            note_ku: None,
        }
    }

    fn repository() -> FakeWaterPlanRepository {
        FakeWaterPlanRepository::holding(vec![an_entry(
            "2026-27",
            "makhmur",
            90.0,
            Some("dukan"),
            Some(40.0),
            true,
        )])
    }

    #[tokio::test]
    async fn replaces_the_whole_line_of_that_season_and_zone() {
        let repository = repository();
        let use_case = UpdateWaterPlanEntryUseCase::new(Arc::new(repository.clone()));

        let updated = use_case
            .execute(&staff(), input("makhmur", Some(12.0)))
            .await
            .expect("entry");

        assert_eq!(updated.need().value(), 75.0);
        assert_eq!(*updated.send_million_m3(), Some(12.0));
        assert!(
            updated.dam_slug().is_none() && !*updated.urgent(),
            "fields left out of the body are cleared, not kept"
        );
        assert_eq!(
            repository.calls(),
            vec![RepositoryCall::Update {
                season: "2026-27".to_string(),
                zone_slug: "makhmur".to_string(),
            }],
            "one update-only write, and no lookup before it"
        );
    }

    #[tokio::test]
    async fn a_zone_not_in_the_plan_is_not_found_and_none_is_created() {
        let repository = repository();
        let use_case = UpdateWaterPlanEntryUseCase::new(Arc::new(repository.clone()));

        let result = use_case.execute(&staff(), input("koya", None)).await;

        assert!(matches!(
            result,
            Err(AppError::EntryNotFound { zone_slug, .. }) if zone_slug == "koya"
        ));
        assert!(
            !repository.calls().iter().any(|call| matches!(
                call,
                RepositoryCall::Upsert { .. } | RepositoryCall::Create { .. }
            )),
            "an update must never fall back to creating"
        );
    }

    #[tokio::test]
    async fn a_negative_volume_is_refused_and_not_written() {
        let repository = repository();
        let use_case = UpdateWaterPlanEntryUseCase::new(Arc::new(repository.clone()));

        let result = use_case
            .execute(&staff(), input("makhmur", Some(-1.0)))
            .await;

        assert!(matches!(result, Err(AppError::Water(_))));
        assert!(repository.calls().is_empty());
    }

    #[tokio::test]
    async fn a_repository_failure_surfaces() {
        let use_case =
            UpdateWaterPlanEntryUseCase::new(Arc::new(FakeWaterPlanRepository::failing()));

        assert!(
            use_case
                .execute(&staff(), input("makhmur", None))
                .await
                .is_err()
        );
    }
}
