use std::sync::Arc;

use crate::{
    app::Pagination,
    features::farms::{
        app::{AppError, FarmRepository},
        domain::{FarmFilter, FarmOrder, OwnedFarmSummary},
    },
};

pub struct ListAllFarmsInput {
    /// Which farms to list. The default lists every farmer's.
    pub filter: FarmFilter,
    pub order: FarmOrder,
    pub pagination: Pagination,
}

/// The dashboard's listing: every farmer's farms, with whose each one is.
/// The farmer app lists through `ListFarmsUseCase`, scoped to its own phone.
pub struct ListAllFarmsUseCase {
    repository: Arc<dyn FarmRepository>,
}

impl ListAllFarmsUseCase {
    pub fn new(repository: Arc<dyn FarmRepository>) -> Self {
        Self { repository }
    }

    /// Returns the page and how many farms match in all.
    pub async fn execute(
        &self,
        input: ListAllFarmsInput,
    ) -> Result<(Vec<OwnedFarmSummary>, u64), AppError> {
        let (farms, count) = self
            .repository
            .find_page(&input.filter, input.order, &input.pagination)
            .await?;

        tracing::debug!(returned = farms.len(), count, "farms listed for staff");

        Ok((farms, count))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        features::farms::{
            app::testing::{FakeFarmRepository, OWNER, RepositoryCall, a_farm},
            domain::{AreaFilter, Crop, FarmSearch, FarmSortKey, PlantedCrop, SortDirection},
        },
        shared::Phone,
    };

    fn input(owner: Option<&str>) -> ListAllFarmsInput {
        ListAllFarmsInput {
            filter: FarmFilter {
                owner: owner.map(|phone| Phone::new(phone.to_string()).expect("phone")),
                ..FarmFilter::default()
            },
            order: FarmOrder::default(),
            pagination: Pagination::new(2, 20),
        }
    }

    #[tokio::test]
    async fn lists_farms_of_every_owner_with_whose_they_are() {
        let repository = FakeFarmRepository::holding(a_farm());
        let use_case = ListAllFarmsUseCase::new(Arc::new(repository.clone()));

        let (farms, count) = use_case.execute(input(None)).await.expect("listing");

        assert_eq!(count, 1);
        assert_eq!(farms[0].owner().as_str(), OWNER);
        assert_eq!(
            repository.calls(),
            vec![RepositoryCall::FindPage {
                filter: FarmFilter::default(),
                order: FarmOrder::default(),
                page: 2,
            }]
        );
    }

    #[tokio::test]
    async fn a_phone_narrows_the_listing_to_that_farmer() {
        let repository = FakeFarmRepository::holding(a_farm());
        let use_case = ListAllFarmsUseCase::new(Arc::new(repository.clone()));

        let (farms, _) = use_case
            .execute(input(Some("+9647509999999")))
            .await
            .expect("listing");

        assert!(farms.is_empty());
        assert!(matches!(
            repository.calls().as_slice(),
            [RepositoryCall::FindPage { filter, page: 2, .. }]
                if filter.owner.as_ref().map(Phone::as_str) == Some("+9647509999999")
        ));
    }

    #[tokio::test]
    async fn every_filter_and_the_order_reach_the_repository_as_given() {
        let repository = FakeFarmRepository::holding(a_farm());
        let use_case = ListAllFarmsUseCase::new(Arc::new(repository.clone()));

        let filter = FarmFilter {
            owner: None,
            governorate: Some(AreaFilter::new("Sulaymaniyah".to_string()).expect("filter")),
            zone: Some(AreaFilter::new("chamchamal".to_string()).expect("filter")),
            sub_zone: Some(AreaFilter::Unknown),
            crop: Some(PlantedCrop::new(Crop::of("wheat")).expect("crop")),
            search: Some(FarmSearch::new("upper".to_string()).expect("search")),
        };
        let order = FarmOrder {
            key: FarmSortKey::AreaDunam,
            direction: SortDirection::Ascending,
        };

        use_case
            .execute(ListAllFarmsInput {
                filter: filter.clone(),
                order,
                pagination: Pagination::new(1, 20),
            })
            .await
            .expect("listing");

        assert_eq!(
            repository.calls(),
            vec![RepositoryCall::FindPage {
                filter,
                order,
                page: 1,
            }]
        );
    }

    #[tokio::test]
    async fn a_repository_failure_surfaces() {
        let use_case = ListAllFarmsUseCase::new(Arc::new(FakeFarmRepository::failing()));

        assert!(use_case.execute(input(None)).await.is_err());
    }
}
