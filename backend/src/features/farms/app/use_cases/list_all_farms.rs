use std::sync::Arc;

use crate::{
    app::Pagination,
    features::farms::{
        app::{AppError, FarmRepository},
        domain::OwnedFarmSummary,
    },
    shared::Phone,
};

pub struct ListAllFarmsInput {
    /// Only this farmer's farms, when given.
    pub owner: Option<Phone>,
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
            .find_page(input.owner.as_ref(), &input.pagination)
            .await?;

        tracing::debug!(returned = farms.len(), count, "farms listed for staff");

        Ok((farms, count))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::features::farms::app::testing::{FakeFarmRepository, OWNER, RepositoryCall, a_farm};

    fn input(owner: Option<&str>) -> ListAllFarmsInput {
        ListAllFarmsInput {
            owner: owner.map(|phone| Phone::new(phone.to_string()).expect("phone")),
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
                owner: None,
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
        assert_eq!(
            repository.calls(),
            vec![RepositoryCall::FindPage {
                owner: Some("+9647509999999".to_string()),
                page: 2,
            }]
        );
    }

    #[tokio::test]
    async fn a_repository_failure_surfaces() {
        let use_case = ListAllFarmsUseCase::new(Arc::new(FakeFarmRepository::failing()));

        assert!(use_case.execute(input(None)).await.is_err());
    }
}
