use std::sync::Arc;

use chrono::Utc;

use crate::{
    app::AppError as GlobalAppError,
    features::farms::{
        app::{AppError, AreaDirectory, FarmRepository, PublicTotalsSwitch},
        domain::{AreaFilter, AreaLevel, FarmFilter, FarmStats, PlantedCrop},
    },
};

#[derive(Default)]
pub struct ViewFarmStatsInput {
    pub governorate: Option<AreaFilter>,
    pub zone: Option<AreaFilter>,
    /// Only the farms growing this crop, and of their crops only this one.
    pub crop: Option<PlantedCrop>,
}

/// Farms, farmers and land added up for the dashboard's reports and charts,
/// down to sub-zones. The adding is done by the database; the names of the
/// areas come from the zones feature.
pub struct ViewFarmStatsUseCase {
    repository: Arc<dyn FarmRepository>,
    areas: Arc<dyn AreaDirectory>,
}

impl ViewFarmStatsUseCase {
    pub fn new(repository: Arc<dyn FarmRepository>, areas: Arc<dyn AreaDirectory>) -> Self {
        Self { repository, areas }
    }

    pub async fn execute(&self, input: ViewFarmStatsInput) -> Result<FarmStats, AppError> {
        let filter = FarmFilter {
            governorate: input.governorate,
            zone: input.zone,
            crop: input.crop,
            ..FarmFilter::default()
        };

        let (counts, crop_sums) = self
            .repository
            .sum_by_area(&filter, AreaLevel::SubZone)
            .await?;
        let names = self.areas.names().await?;

        Ok(FarmStats::assemble(counts, crop_sums, &names, Utc::now()))
    }
}

/// The same totals for the public View page: nothing finer than a zone,
/// because a sub-zone with one or two farms would point at a farmer.
pub struct ViewPublicFarmStatsUseCase {
    repository: Arc<dyn FarmRepository>,
    areas: Arc<dyn AreaDirectory>,
    /// Off: the totals are not public, and asking for them finds nothing.
    switch: Arc<dyn PublicTotalsSwitch>,
}

impl ViewPublicFarmStatsUseCase {
    pub fn new(
        repository: Arc<dyn FarmRepository>,
        areas: Arc<dyn AreaDirectory>,
        switch: Arc<dyn PublicTotalsSwitch>,
    ) -> Self {
        Self {
            repository,
            areas,
            switch,
        }
    }

    pub async fn execute(&self) -> Result<FarmStats, AppError> {
        if !self.switch.is_on().await? {
            return Err(GlobalAppError::NotFound.into());
        }

        // The sub-zone sums are not even asked for, so they cannot leak
        // through a later change to what the route writes out.
        let (counts, crop_sums) = self
            .repository
            .sum_by_area(&FarmFilter::default(), AreaLevel::Zone)
            .await?;
        let names = self.areas.names().await?;

        Ok(FarmStats::assemble(counts, crop_sums, &names, Utc::now()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::features::farms::{
        app::testing::{FakeAreaDirectory, FakeFarmRepository, FakeTotalsSwitch, RepositoryCall},
        domain::{AreaCount, AreaCropSum, AreaKey, Crop},
    };

    fn chamchamal(sub_zone: Option<&str>) -> AreaKey {
        AreaKey {
            governorate: Some("Sulaymaniyah".to_string()),
            zone_slug: Some("chamchamal".to_string()),
            sub_zone_slug: sub_zone.map(str::to_string),
        }
    }

    fn count(level: AreaLevel, key: AreaKey) -> AreaCount {
        AreaCount {
            level,
            key,
            farms: 2,
            farmers: 1,
            area_m2: 25_000.0,
            latest_change: None,
        }
    }

    /// Two farms of one farmer in Sangaw, six dunams of wheat between them.
    fn repository() -> FakeFarmRepository {
        FakeFarmRepository::summing(
            vec![
                count(AreaLevel::Region, AreaKey::default()),
                count(AreaLevel::Zone, chamchamal(None)),
                count(AreaLevel::SubZone, chamchamal(Some("sangaw"))),
            ],
            vec![AreaCropSum {
                level: AreaLevel::Region,
                key: AreaKey::default(),
                crop: Crop::of("wheat"),
                inside_pct: 15_000.0,
                farms: 2,
                farmers: 1,
            }],
        )
    }

    #[tokio::test]
    async fn the_dashboard_totals_go_down_to_sub_zones_and_carry_the_zones_names() {
        let repository = repository();
        let areas = FakeAreaDirectory::new();
        let use_case =
            ViewFarmStatsUseCase::new(Arc::new(repository.clone()), Arc::new(areas.clone()));

        let stats = use_case
            .execute(ViewFarmStatsInput::default())
            .await
            .expect("stats");

        assert_eq!(stats.totals().farms(), 2);
        assert_eq!(stats.totals().farmers(), 1);
        assert_eq!(stats.totals().dunam(), 10.0);
        assert_eq!(stats.by_zone()[0].name_ku(), "چەمچەماڵ");
        assert_eq!(stats.by_sub_zone()[0].name_en(), "Sangaw");
        assert_eq!(stats.by_crop()[0].dunam(), 6.0);
        assert_eq!(
            repository.calls(),
            vec![RepositoryCall::SumByArea {
                filter: FarmFilter::default(),
                deepest: AreaLevel::SubZone,
            }]
        );
        assert_eq!(areas.asked(), 1);
    }

    #[tokio::test]
    async fn the_filters_reach_the_repository_and_nothing_else_is_filtered() {
        let repository = repository();
        let use_case = ViewFarmStatsUseCase::new(
            Arc::new(repository.clone()),
            Arc::new(FakeAreaDirectory::new()),
        );

        let governorate = AreaFilter::new("Sulaymaniyah".to_string()).expect("filter");
        let zone = AreaFilter::new("chamchamal".to_string()).expect("filter");
        let crop = PlantedCrop::new(Crop::of("wheat")).expect("crop");

        use_case
            .execute(ViewFarmStatsInput {
                governorate: Some(governorate.clone()),
                zone: Some(zone.clone()),
                crop: Some(crop),
            })
            .await
            .expect("stats");

        assert_eq!(
            repository.calls(),
            vec![RepositoryCall::SumByArea {
                filter: FarmFilter {
                    owner: None,
                    governorate: Some(governorate),
                    zone: Some(zone),
                    sub_zone: None,
                    crop: Some(crop),
                    search: None,
                },
                deepest: AreaLevel::SubZone,
            }]
        );
    }

    #[tokio::test]
    async fn the_public_totals_never_ask_for_anything_finer_than_a_zone() {
        let repository = repository();
        let use_case = ViewPublicFarmStatsUseCase::new(
            Arc::new(repository.clone()),
            Arc::new(FakeAreaDirectory::new()),
            Arc::new(FakeTotalsSwitch(true)),
        );

        let stats = use_case.execute().await.expect("stats");

        assert_eq!(stats.totals().farms(), 2);
        assert_eq!(
            repository.calls(),
            vec![RepositoryCall::SumByArea {
                filter: FarmFilter::default(),
                deepest: AreaLevel::Zone,
            }]
        );
    }

    #[tokio::test]
    async fn switched_off_the_public_totals_are_not_found_and_nothing_is_read() {
        let repository = repository();
        let areas = FakeAreaDirectory::new();
        let use_case = ViewPublicFarmStatsUseCase::new(
            Arc::new(repository.clone()),
            Arc::new(areas.clone()),
            Arc::new(FakeTotalsSwitch(false)),
        );

        assert!(matches!(
            use_case.execute().await,
            Err(AppError::GlobalAppError(GlobalAppError::NotFound))
        ));
        assert!(repository.calls().is_empty());
        assert_eq!(areas.asked(), 0);
    }

    #[tokio::test]
    async fn a_failure_of_the_repository_or_of_the_names_surfaces() {
        let failing_repository = ViewFarmStatsUseCase::new(
            Arc::new(FakeFarmRepository::failing()),
            Arc::new(FakeAreaDirectory::new()),
        );
        let failing_names = ViewFarmStatsUseCase::new(
            Arc::new(repository()),
            Arc::new(FakeAreaDirectory::failing()),
        );

        assert!(
            failing_repository
                .execute(ViewFarmStatsInput::default())
                .await
                .is_err()
        );
        assert!(
            failing_names
                .execute(ViewFarmStatsInput::default())
                .await
                .is_err()
        );
    }
}
