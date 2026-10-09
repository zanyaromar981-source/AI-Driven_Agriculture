use std::sync::Arc;

use crate::features::alwa::app::{AlwaRepository, AppError};

pub struct DeleteListingUseCase {
    repository: Arc<dyn AlwaRepository>,
}

impl DeleteListingUseCase {
    pub fn new(repository: Arc<dyn AlwaRepository>) -> Self {
        Self { repository }
    }

    /// A staff member removes a listing and its offers. A listing that was
    /// sold is a deal on record and stays. Removing one that is already gone
    /// succeeds: gone is what was asked.
    pub async fn execute(&self, staff_id: i32, id: i32) -> Result<(), AppError> {
        let removed = self
            .repository
            .delete_listing(id)
            .await
            .inspect_err(|error| {
                tracing::info!(staff_id, listing_id = id, %error, "listing delete refused");
            })?;

        tracing::info!(
            staff_id,
            listing_id = id,
            removed,
            "listing deleted by staff"
        );

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::features::alwa::{
        app::testing::{
            BUYER, FakeAlwaRepository, RepositoryCall, a_sold_listing, an_open_listing,
            an_open_offer,
        },
        domain::AlwaError,
    };

    #[tokio::test]
    async fn removes_the_listing_and_its_offers_and_no_other() {
        let listing = an_open_listing(7);
        let other = an_open_listing(8);
        let repository = FakeAlwaRepository::new()
            .with_offer(an_open_offer(1, &listing, BUYER, 900))
            .with_offer(an_open_offer(2, &other, BUYER, 900))
            .with_listing(listing)
            .with_listing(other);
        let use_case = DeleteListingUseCase::new(Arc::new(repository.clone()));

        use_case.execute(9, 7).await.expect("delete");

        assert!(repository.stored_listing(7).is_none());
        assert!(repository.stored_listing(8).is_some());
        assert_eq!(repository.stored_offers().len(), 1);
        assert_eq!(
            repository.calls(),
            vec![RepositoryCall::DeleteListing { id: 7 }],
            "the delete decides, with no lookup before it"
        );
    }

    #[tokio::test]
    async fn deleting_again_succeeds() {
        let repository = FakeAlwaRepository::new().with_listing(an_open_listing(7));
        let use_case = DeleteListingUseCase::new(Arc::new(repository.clone()));

        use_case.execute(9, 7).await.expect("first");

        assert!(use_case.execute(9, 7).await.is_ok());
    }

    #[tokio::test]
    async fn a_listing_with_a_deal_stays_with_all_its_offers() {
        let (sold, offers) = a_sold_listing(7);
        let repository = offers
            .into_iter()
            .fold(FakeAlwaRepository::new(), FakeAlwaRepository::with_offer)
            .with_listing(sold);
        let use_case = DeleteListingUseCase::new(Arc::new(repository.clone()));

        assert!(matches!(
            use_case.execute(9, 7).await,
            Err(AppError::Alwa(AlwaError::ListingHasDeal))
        ));
        assert!(repository.stored_listing(7).is_some());
        assert_eq!(repository.stored_offers().len(), 2);
    }
}
