use std::sync::Arc;

use async_trait::async_trait;

use crate::{
    app::AppError as GlobalAppError,
    features::{
        farmers::{
            app::{AppError, FarmHoldings},
            domain::{CropHolding, FarmHolding},
        },
        farms::app::FarmRepository,
    },
    shared::Phone,
};

/// Lists a farmer's farms for the support letter by asking the farms
/// feature through its own repository port.
#[derive(Debug)]
pub struct FarmsFeatureFarmHoldings {
    farms: Arc<dyn FarmRepository>,
}

impl FarmsFeatureFarmHoldings {
    pub fn new(farms: Arc<dyn FarmRepository>) -> Self {
        Self { farms }
    }
}

#[async_trait]
impl FarmHoldings for FarmsFeatureFarmHoldings {
    async fn of(&self, phone: &Phone) -> Result<Vec<FarmHolding>, AppError> {
        let farms = self.farms.find_all_by_owner(phone).await.map_err(|error| {
            tracing::error!(%error, "listing a farmer's farms for a letter failed");

            AppError::from(GlobalAppError::InternalServerError)
        })?;

        Ok(farms
            .iter()
            .map(|farm| FarmHolding {
                id: *farm.id(),
                name: farm.name().as_str().to_string(),
                // The farms feature does not say where a farm is yet. Until
                // its summary carries a governorate, zone and sub-zone the
                // letter states none, which is the truth; this is the one
                // place to read them from once it does.
                governorate: None,
                zone_slug: None,
                sub_zone_slug: None,
                area_dunam: *farm.area_dunam(),
                crops: farm
                    .crops()
                    .iter()
                    .map(|area| CropHolding {
                        crop: area.crop().into(),
                        dunam: area.dunam(),
                    })
                    .collect(),
            })
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::features::farms::app::testing::{FakeFarmRepository, OWNER, RepositoryCall, a_farm};

    fn phone() -> Phone {
        Phone::new(OWNER.to_string()).expect("phone")
    }

    #[tokio::test]
    async fn lists_the_farms_of_the_given_phone_with_area_and_crops() {
        let farms = FakeFarmRepository::holding(a_farm());
        let holdings = FarmsFeatureFarmHoldings::new(Arc::new(farms.clone()));

        let listed = holdings.of(&phone()).await.expect("farms");

        assert_eq!(listed.len(), 1);
        assert_eq!(listed[0].id, 7);
        assert_eq!(listed[0].name, "Upper field");
        assert!(listed[0].area_dunam > 0.0);
        assert_eq!(listed[0].crops[0].crop, "wheat");
        assert_eq!(
            farms.calls(),
            vec![RepositoryCall::FindAllByOwner {
                owner: OWNER.to_string(),
            }]
        );
    }

    #[tokio::test]
    async fn a_farm_has_no_place_until_the_farms_feature_gives_one() {
        let holdings =
            FarmsFeatureFarmHoldings::new(Arc::new(FakeFarmRepository::holding(a_farm())));

        let listed = holdings.of(&phone()).await.expect("farms");

        assert!(listed[0].governorate.is_none());
        assert!(listed[0].zone_slug.is_none());
        assert!(listed[0].sub_zone_slug.is_none());
    }

    #[tokio::test]
    async fn a_failure_in_the_farms_feature_is_an_error() {
        let holdings = FarmsFeatureFarmHoldings::new(Arc::new(FakeFarmRepository::failing()));

        assert!(holdings.of(&phone()).await.is_err());
    }
}
