use std::sync::Arc;

use async_trait::async_trait;

use crate::{
    app::AppError as GlobalAppError,
    features::{
        doctor::{
            app::{AppError, FarmBriefs},
            domain::FarmBrief,
        },
        farms::{app::FarmRepository, domain::Farm},
    },
    shared::Phone,
};

/// Describes a farm for the Doctor by asking the farms feature through its
/// own repository port, so what "belongs to" means stays in one place.
#[derive(Debug)]
pub struct FarmsFeatureFarmBriefs {
    farms: Arc<dyn FarmRepository>,
}

impl FarmsFeatureFarmBriefs {
    pub fn new(farms: Arc<dyn FarmRepository>) -> Self {
        Self { farms }
    }
}

#[async_trait]
impl FarmBriefs for FarmsFeatureFarmBriefs {
    async fn brief_for(&self, farm_id: i32, owner: &Phone) -> Result<Option<FarmBrief>, AppError> {
        let farm = self
            .farms
            .find_by_id_and_owner(farm_id, owner)
            .await
            .map_err(|error| {
                tracing::error!(%error, farm_id, "reading the farm for the Doctor failed");

                GlobalAppError::InternalServerError
            })?;

        Ok(farm.map(|farm| brief_of(farm_id, &farm)))
    }
}

/// The centre is the mean of the walked corners and the area the exact one
/// inside the outline, the same figures `GET /v1/farms/{id}` shows.
fn brief_of(farm_id: i32, farm: &Farm) -> FarmBrief {
    let (lat, lon) = farm.outline().centroid();

    FarmBrief::rehydrate(
        farm_id,
        farm.name().into(),
        lat,
        lon,
        farm.outline().area_m2(),
        farm.crop_areas()
            .iter()
            .map(|area| String::from(area.crop()))
            .collect(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::features::farms::app::testing::{
        FakeFarmRepository, OWNER, RepositoryCall, a_farm, an_outline,
    };

    fn phone() -> Phone {
        Phone::new(OWNER.to_string()).expect("phone")
    }

    #[tokio::test]
    async fn the_brief_carries_the_farms_centre_exact_area_and_crops() {
        let briefs = FarmsFeatureFarmBriefs::new(Arc::new(FakeFarmRepository::holding(a_farm())));

        let brief = briefs
            .brief_for(7, &phone())
            .await
            .expect("answer")
            .expect("brief");

        let corners = an_outline().points().to_vec();
        let mean = |pick: fn(&crate::features::farms::domain::Point) -> f64| {
            corners.iter().map(pick).sum::<f64>() / corners.len() as f64
        };

        assert_eq!(*brief.id(), 7);
        assert_eq!(brief.name(), "Upper field");
        assert_eq!(*brief.lat(), mean(|point| point.lat()));
        assert_eq!(*brief.lon(), mean(|point| point.lon()));
        assert_eq!(
            *brief.area_m2(),
            an_outline().area_m2(),
            "the exact area inside the outline, not a cell count"
        );
        assert_eq!(brief.crops(), &vec!["wheat".to_string()]);
    }

    #[tokio::test]
    async fn the_farm_is_looked_up_for_the_owner_never_by_id_alone() {
        let farms = FakeFarmRepository::holding(a_farm());
        let briefs = FarmsFeatureFarmBriefs::new(Arc::new(farms.clone()));

        briefs.brief_for(7, &phone()).await.expect("answer");

        assert_eq!(
            farms.calls(),
            vec![RepositoryCall::FindByIdAndOwner {
                id: 7,
                owner: OWNER.to_string(),
            }]
        );
    }

    #[tokio::test]
    async fn a_farm_the_farms_feature_does_not_find_has_no_brief() {
        let briefs = FarmsFeatureFarmBriefs::new(Arc::new(FakeFarmRepository::new()));

        assert!(
            briefs
                .brief_for(7, &phone())
                .await
                .expect("answer")
                .is_none()
        );
    }

    #[tokio::test]
    async fn a_failure_in_the_farms_feature_is_an_error_not_a_missing_farm() {
        let briefs = FarmsFeatureFarmBriefs::new(Arc::new(FakeFarmRepository::failing()));

        assert!(briefs.brief_for(7, &phone()).await.is_err());
    }
}
