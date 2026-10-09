use std::sync::Arc;

use async_trait::async_trait;

use crate::{
    app::{AppError as GlobalAppError, Pagination},
    features::{
        farms::{
            app::FarmRepository,
            domain::{AreaFilter, FarmFilter, FarmOrder},
        },
        messages::{
            app::{AppError, MessageFarmDirectory},
            domain::FarmCard,
        },
    },
    shared::Phone,
};

/// Finds the farms messages are about by asking the farms feature through
/// its own repository port. Only the id and the name cross over.
#[derive(Debug)]
pub struct FarmsFeatureMessageFarmDirectory {
    farms: Arc<dyn FarmRepository>,
}

impl FarmsFeatureMessageFarmDirectory {
    pub fn new(farms: Arc<dyn FarmRepository>) -> Self {
        Self { farms }
    }
}

/// The inbox filter reads at most this many pages of 100 farms for a place.
const MAX_PLACE_PAGES: u64 = 100;

fn failed(error: impl std::fmt::Display, doing: &str) -> AppError {
    tracing::error!(%error, "{doing} for messages failed");

    GlobalAppError::InternalServerError.into()
}

#[async_trait]
impl MessageFarmDirectory for FarmsFeatureMessageFarmDirectory {
    async fn is_owned_by(&self, farm_id: i32, phone: &Phone) -> Result<bool, AppError> {
        // The owner's list is short and carries no cells, where reading the
        // one farm would load every cell it has.
        let farms = self
            .farms
            .find_all_by_owner(phone)
            .await
            .map_err(|error| failed(error, "checking farm ownership"))?;

        Ok(farms.iter().any(|farm| *farm.id() == farm_id))
    }

    async fn cards_of(&self, farm_ids: &[i32]) -> Result<Vec<FarmCard>, AppError> {
        if farm_ids.is_empty() {
            return Ok(Vec::new());
        }

        let farms = self
            .farms
            .find_summaries_by_ids(farm_ids)
            .await
            .map_err(|error| failed(error, "reading farm names"))?;

        Ok(farms
            .iter()
            .map(|farm| {
                FarmCard::rehydrate(
                    *farm.id(),
                    farm.name().as_str().to_string(),
                    farm.place()
                        .as_ref()
                        .map(|place| place.governorate().clone()),
                    farm.place().as_ref().map(|place| place.zone_slug().clone()),
                )
            })
            .collect())
    }

    async fn ids_in_place(
        &self,
        governorate: Option<&str>,
        zone_slug: Option<&str>,
    ) -> Result<Vec<i32>, AppError> {
        let area = |value: Option<&str>| {
            value
                .map(|value| AreaFilter::new(value.to_string()))
                .transpose()
        };
        // A value the farms feature would refuse names no place, so no farm
        // is in it.
        let (Ok(governorate), Ok(zone)) = (area(governorate), area(zone_slug)) else {
            return Ok(Vec::new());
        };
        let filter = FarmFilter {
            governorate,
            zone,
            ..FarmFilter::default()
        };

        let mut ids = Vec::new();

        for page in 1..=MAX_PLACE_PAGES {
            let (farms, total) = self
                .farms
                .find_page(&filter, FarmOrder::default(), &Pagination::new(page, 100))
                .await
                .map_err(|error| failed(error, "listing the farms of a place"))?;

            ids.extend(farms.iter().map(|farm| *farm.summary().id()));

            if farms.is_empty() || ids.len() as u64 >= total {
                return Ok(ids);
            }
        }

        tracing::warn!(
            farms = ids.len(),
            "a place holds more farms than the inbox filter reads; later ones are left out"
        );

        Ok(ids)
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
    async fn a_farm_in_the_owners_list_is_theirs() {
        let farms = FakeFarmRepository::holding(a_farm());
        let directory = FarmsFeatureMessageFarmDirectory::new(Arc::new(farms.clone()));

        assert!(directory.is_owned_by(7, &phone()).await.expect("answer"));
        assert!(!directory.is_owned_by(8, &phone()).await.expect("answer"));
        assert_eq!(
            farms.calls()[0],
            RepositoryCall::FindAllByOwner {
                owner: OWNER.to_string()
            },
            "the farms are asked for by the owner's phone, never by id alone"
        );
    }

    #[tokio::test]
    async fn an_owner_with_no_farms_owns_none() {
        let directory = FarmsFeatureMessageFarmDirectory::new(Arc::new(FakeFarmRepository::new()));

        assert!(!directory.is_owned_by(7, &phone()).await.expect("answer"));
    }

    #[tokio::test]
    async fn a_card_is_the_farms_id_and_name_and_no_place_when_the_farm_has_none() {
        let farms = FakeFarmRepository::holding(a_farm());
        let directory = FarmsFeatureMessageFarmDirectory::new(Arc::new(farms.clone()));

        let cards = directory.cards_of(&[7, 8]).await.expect("cards");

        assert_eq!(
            cards,
            vec![FarmCard::rehydrate(
                7,
                "Upper field".to_string(),
                None,
                None
            )],
            "the farm with id 8 is gone and is simply absent"
        );
        assert_eq!(
            farms.calls(),
            vec![RepositoryCall::FindSummariesByIds { ids: vec![7, 8] }]
        );
    }

    #[tokio::test]
    async fn the_farms_of_a_place_are_asked_from_the_farms_feature() {
        let farms = FakeFarmRepository::holding(a_farm());
        let directory = FarmsFeatureMessageFarmDirectory::new(Arc::new(farms.clone()));

        let ids = directory
            .ids_in_place(Some("Sulaymaniyah"), Some("chamchamal"))
            .await
            .expect("ids");

        assert_eq!(ids, vec![7]);
        assert_eq!(
            farms.calls(),
            vec![RepositoryCall::FindPage {
                filter: FarmFilter {
                    governorate: Some(AreaFilter::new("sulaymaniyah".to_string()).expect("area")),
                    zone: Some(AreaFilter::new("chamchamal".to_string()).expect("area")),
                    ..FarmFilter::default()
                },
                order: FarmOrder::default(),
                page: 1,
            }],
            "one page is enough when it holds every farm of the place"
        );
    }

    #[tokio::test]
    async fn a_place_name_the_farms_feature_refuses_holds_no_farm() {
        let farms = FakeFarmRepository::holding(a_farm());
        let directory = FarmsFeatureMessageFarmDirectory::new(Arc::new(farms.clone()));

        let ids = directory.ids_in_place(Some(""), None).await.expect("ids");

        assert!(ids.is_empty());
        assert!(farms.calls().is_empty());
    }

    #[tokio::test]
    async fn a_failure_in_the_farms_feature_is_an_error_not_a_no() {
        let directory =
            FarmsFeatureMessageFarmDirectory::new(Arc::new(FakeFarmRepository::failing()));

        assert!(directory.is_owned_by(7, &phone()).await.is_err());
        assert!(directory.cards_of(&[7]).await.is_err());
        assert!(
            directory
                .ids_in_place(None, Some("chamchamal"))
                .await
                .is_err()
        );
    }
}
