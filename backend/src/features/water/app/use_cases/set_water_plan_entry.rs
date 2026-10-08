use std::sync::Arc;

use crate::features::water::{
    app::{AppError, WaterPlanRepository},
    domain::{DamSlug, Need, Note, Season, WaterPlanEntry, ZoneSlug},
};

pub struct SetWaterPlanEntryInput {
    pub season: Season,
    pub zone_slug: ZoneSlug,
    pub need: Need,
    pub dam_slug: Option<DamSlug>,
    pub send_million_m3: Option<f64>,
    pub urgent: bool,
    pub note_en: Option<Note>,
    pub note_ku: Option<Note>,
}

pub struct SetWaterPlanEntryUseCase {
    repository: Arc<dyn WaterPlanRepository>,
}

impl SetWaterPlanEntryUseCase {
    pub fn new(repository: Arc<dyn WaterPlanRepository>) -> Self {
        Self { repository }
    }

    /// Stores one zone's line in a season's plan. Sending the same season
    /// and zone again replaces the earlier line as a whole.
    pub async fn execute(&self, input: SetWaterPlanEntryInput) -> Result<WaterPlanEntry, AppError> {
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

        let stored = self.repository.upsert(&entry).await?;

        tracing::info!(
            season = stored.season().as_str(),
            zone = stored.zone_slug().as_str(),
            need = stored.need().value(),
            dam = stored.dam_slug().as_ref().map(DamSlug::as_str),
            send_million_m3 = *stored.send_million_m3(),
            urgent = *stored.urgent(),
            "water plan entry set"
        );

        Ok(stored)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::features::water::app::testing::{
        FakeWaterPlanRepository, RepositoryCall, dam_slug, season, zone_slug,
    };

    fn input(send_million_m3: Option<f64>) -> SetWaterPlanEntryInput {
        SetWaterPlanEntryInput {
            season: season("2026-27"),
            zone_slug: zone_slug("makhmur"),
            need: Need::new(90.0).expect("need"),
            dam_slug: Some(dam_slug("dukan")),
            send_million_m3,
            urgent: true,
            note_en: Some(Note::new("Canal under repair".to_string()).expect("note")),
            note_ku: Some(Note::new("جۆگەکە چاک دەکرێتەوە".to_string()).expect("note")),
        }
    }

    #[tokio::test]
    async fn stores_the_entry_under_its_season_and_zone() {
        let repository = FakeWaterPlanRepository::new();
        let use_case = SetWaterPlanEntryUseCase::new(Arc::new(repository.clone()));

        let stored = use_case.execute(input(Some(40.0))).await.expect("entry");

        assert!(stored.id().is_some(), "the stored entry comes back");
        assert_eq!(stored.need().value(), 90.0);
        assert_eq!(
            stored.dam_slug().as_ref().map(DamSlug::as_str),
            Some("dukan")
        );
        assert_eq!(*stored.send_million_m3(), Some(40.0));
        assert!(*stored.urgent());
        assert_eq!(
            stored.note_ku().as_ref().map(Note::as_str),
            Some("جۆگەکە چاک دەکرێتەوە")
        );
        assert_eq!(
            repository.calls(),
            vec![RepositoryCall::Upsert {
                season: "2026-27".to_string(),
                zone_slug: "makhmur".to_string(),
            }],
            "one write, keyed by the season and the zone"
        );
    }

    #[tokio::test]
    async fn a_negative_volume_is_not_written() {
        let repository = FakeWaterPlanRepository::new();
        let use_case = SetWaterPlanEntryUseCase::new(Arc::new(repository.clone()));

        let result = use_case.execute(input(Some(-1.0))).await;

        assert!(matches!(result, Err(AppError::Water(_))));
        assert!(repository.calls().is_empty());
    }

    #[tokio::test]
    async fn a_repository_failure_surfaces() {
        let use_case = SetWaterPlanEntryUseCase::new(Arc::new(FakeWaterPlanRepository::failing()));

        assert!(use_case.execute(input(None)).await.is_err());
    }
}
