use std::sync::Arc;

use crate::{
    app::StaffContext,
    features::water::{
        app::{AppError, WaterPlanRepository, use_cases::SetWaterPlanEntryInput},
        domain::{DamSlug, WaterPlanEntry},
    },
};

pub struct CreateWaterPlanEntryUseCase {
    repository: Arc<dyn WaterPlanRepository>,
}

impl CreateWaterPlanEntryUseCase {
    pub fn new(repository: Arc<dyn WaterPlanRepository>) -> Self {
        Self { repository }
    }

    /// Adds a zone a staff member put into a season's plan. Whether the
    /// zone is already in it is not looked up first: the unique index
    /// answers that when the entry is stored, so of two copies sent at the
    /// same moment exactly one is created.
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

        let created =
            self.repository
                .create(&entry)
                .await?
                .ok_or_else(|| AppError::EntryAlreadyExists {
                    season: String::from(entry.season()),
                    zone_slug: String::from(entry.zone_slug()),
                })?;

        tracing::info!(
            staff_id = *actor.staff_id(),
            season = created.season().as_str(),
            zone = created.zone_slug().as_str(),
            need = created.need().value(),
            dam = created.dam_slug().as_ref().map(DamSlug::as_str),
            send_million_m3 = *created.send_million_m3(),
            urgent = *created.urgent(),
            "water plan entry created from the dashboard"
        );

        Ok(created)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::features::water::{
        app::testing::{
            FakeWaterPlanRepository, RepositoryCall, an_entry, dam_slug, season, staff, zone_slug,
        },
        domain::Need,
    };

    fn input(zone: &str, send_million_m3: Option<f64>) -> SetWaterPlanEntryInput {
        SetWaterPlanEntryInput {
            season: season("2026-27"),
            zone_slug: zone_slug(zone),
            need: Need::new(90.0).expect("need"),
            dam_slug: Some(dam_slug("dukan")),
            send_million_m3,
            urgent: true,
            note_en: None,
            note_ku: None,
        }
    }

    #[tokio::test]
    async fn creates_the_entry_under_its_season_and_zone() {
        let repository = FakeWaterPlanRepository::new();
        let use_case = CreateWaterPlanEntryUseCase::new(Arc::new(repository.clone()));

        let created = use_case
            .execute(&staff(), input("makhmur", Some(40.0)))
            .await
            .expect("entry");

        assert!(created.id().is_some(), "the stored entry comes back");
        assert_eq!(created.need().value(), 90.0);
        assert_eq!(*created.send_million_m3(), Some(40.0));
        assert_eq!(
            repository.calls(),
            vec![RepositoryCall::Create {
                season: "2026-27".to_string(),
                zone_slug: "makhmur".to_string(),
            }],
            "one create-only write, and no lookup before it"
        );
    }

    #[tokio::test]
    async fn a_zone_already_in_the_plan_is_refused_and_left_as_it_was() {
        let repository = FakeWaterPlanRepository::holding(vec![an_entry(
            "2026-27", "makhmur", 50.0, None, None, false,
        )]);
        let use_case = CreateWaterPlanEntryUseCase::new(Arc::new(repository.clone()));

        let result = use_case.execute(&staff(), input("makhmur", None)).await;

        assert!(matches!(
            result,
            Err(AppError::EntryAlreadyExists { season, zone_slug })
                if season == "2026-27" && zone_slug == "makhmur"
        ));
        assert!(
            !repository
                .calls()
                .iter()
                .any(|call| matches!(call, RepositoryCall::Upsert { .. })),
            "a create must never fall back to replacing"
        );
    }

    #[tokio::test]
    async fn a_negative_volume_is_refused_as_on_ingest_and_not_written() {
        let repository = FakeWaterPlanRepository::new();
        let use_case = CreateWaterPlanEntryUseCase::new(Arc::new(repository.clone()));

        let result = use_case
            .execute(&staff(), input("makhmur", Some(-1.0)))
            .await;

        assert!(matches!(result, Err(AppError::Water(_))));
        assert!(repository.calls().is_empty());
    }

    #[tokio::test]
    async fn a_repository_failure_surfaces() {
        let use_case =
            CreateWaterPlanEntryUseCase::new(Arc::new(FakeWaterPlanRepository::failing()));

        assert!(
            use_case
                .execute(&staff(), input("makhmur", None))
                .await
                .is_err()
        );
    }
}
