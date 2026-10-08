use std::sync::Arc;

use chrono::Utc;

use crate::{
    app::AuthContext,
    features::alwa::{
        app::{AlwaRepository, AppError, listing_cards::assemble_cards},
        domain::ListingCard,
    },
};

pub struct ListMyListingsUseCase {
    repository: Arc<dyn AlwaRepository>,
}

impl ListMyListingsUseCase {
    pub fn new(repository: Arc<dyn AlwaRepository>) -> Self {
        Self { repository }
    }

    /// Returns the signed-in phone's listings of every status, newest first,
    /// each with its offers.
    pub async fn execute(&self, auth_context: &AuthContext) -> Result<Vec<ListingCard>, AppError> {
        let listings = self
            .repository
            .find_listings_by_seller(auth_context.user().phone())
            .await?;

        let cards = assemble_cards(self.repository.as_ref(), listings, Utc::now()).await?;

        tracing::debug!(returned = cards.len(), "own listings listed");

        Ok(cards)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::features::alwa::app::testing::{
        BUYER, FakeAlwaRepository, RepositoryCall, SELLER, a_sold_listing, an_open_listing,
        auth_context,
    };

    fn a_seller_with_two_listings() -> FakeAlwaRepository {
        let (sold, offers) = a_sold_listing(7);

        offers.into_iter().fold(
            FakeAlwaRepository::new()
                .with_listing(sold)
                .with_listing(an_open_listing(8)),
            FakeAlwaRepository::with_offer,
        )
    }

    #[tokio::test]
    async fn lists_the_sellers_listings_of_every_status_with_their_offers() {
        let repository = a_seller_with_two_listings();
        let use_case = ListMyListingsUseCase::new(Arc::new(repository.clone()));

        let cards = use_case
            .execute(&auth_context(SELLER))
            .await
            .expect("cards");

        assert_eq!(cards.len(), 2);
        assert_eq!(cards[0].offers().len(), 2);
        assert!(cards[1].offers().is_empty());
        assert_eq!(
            repository.calls()[0],
            RepositoryCall::FindListingsBySeller {
                seller: SELLER.to_string()
            },
            "the seller must be part of the lookup"
        );
    }

    #[tokio::test]
    async fn another_phone_sees_none_of_them() {
        let repository = a_seller_with_two_listings();
        let use_case = ListMyListingsUseCase::new(Arc::new(repository.clone()));

        let cards = use_case.execute(&auth_context(BUYER)).await.expect("cards");

        assert!(cards.is_empty());
    }

    #[tokio::test]
    async fn a_repository_failure_surfaces() {
        let use_case = ListMyListingsUseCase::new(Arc::new(FakeAlwaRepository::failing()));

        assert!(use_case.execute(&auth_context(SELLER)).await.is_err());
    }
}
