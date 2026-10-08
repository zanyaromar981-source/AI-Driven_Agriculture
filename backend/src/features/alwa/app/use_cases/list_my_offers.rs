use std::sync::Arc;

use chrono::Utc;

use crate::{
    app::AuthContext,
    features::alwa::{
        app::{AlwaRepository, AppError},
        domain::PlacedOffer,
    },
};

pub struct ListMyOffersUseCase {
    repository: Arc<dyn AlwaRepository>,
}

impl ListMyOffersUseCase {
    pub fn new(repository: Arc<dyn AlwaRepository>) -> Self {
        Self { repository }
    }

    /// Returns the signed-in phone's offers of every status, newest first,
    /// each with the listing it was made on.
    pub async fn execute(&self, auth_context: &AuthContext) -> Result<Vec<PlacedOffer>, AppError> {
        let offers = self
            .repository
            .find_offers_by_buyer(auth_context.user().phone())
            .await?;

        if offers.is_empty() {
            return Ok(Vec::new());
        }

        let mut listing_ids: Vec<i32> = offers.iter().map(|offer| *offer.listing_id()).collect();
        listing_ids.sort_unstable();
        listing_ids.dedup();

        let listings = self.repository.find_listings_by_ids(&listing_ids).await?;
        let now = Utc::now();

        let placed: Vec<PlacedOffer> = offers
            .into_iter()
            .filter_map(|offer| {
                let listing = listings.iter().find(|listing| offer.is_on(listing))?;

                Some(PlacedOffer::new(offer, listing.clone(), now))
            })
            .collect();

        tracing::debug!(returned = placed.len(), "own offers listed");

        Ok(placed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::features::alwa::{
        app::testing::{
            BUYER, FakeAlwaRepository, OTHER_BUYER, RepositoryCall, SELLER, a_sold_listing,
            an_open_listing, an_open_offer, auth_context,
        },
        domain::ListingStatus,
    };

    fn a_buyer_with_two_offers() -> FakeAlwaRepository {
        let (sold, offers) = a_sold_listing(7);
        let open = an_open_listing(8);

        offers.into_iter().fold(
            FakeAlwaRepository::new()
                .with_offer(an_open_offer(3, &open, BUYER, 990))
                .with_listing(sold)
                .with_listing(open),
            FakeAlwaRepository::with_offer,
        )
    }

    #[tokio::test]
    async fn lists_only_the_buyers_own_offers_each_with_its_listing() {
        let repository = a_buyer_with_two_offers();
        let use_case = ListMyOffersUseCase::new(Arc::new(repository.clone()));

        let placed = use_case
            .execute(&auth_context(BUYER))
            .await
            .expect("offers");

        assert_eq!(placed.len(), 2);
        assert!(
            placed
                .iter()
                .all(|placed| placed.offer().buyer_phone().as_str() == BUYER)
        );
        assert!(
            placed
                .iter()
                .all(|placed| placed.offer().is_on(placed.listing()))
        );
        assert_eq!(
            repository.calls(),
            vec![
                RepositoryCall::FindOffersByBuyer {
                    buyer: BUYER.to_string()
                },
                RepositoryCall::FindListingsByIds { ids: vec![7, 8] },
            ]
        );
    }

    #[tokio::test]
    async fn each_offer_carries_the_status_of_its_listing() {
        let repository = a_buyer_with_two_offers();
        let use_case = ListMyOffersUseCase::new(Arc::new(repository.clone()));

        let placed = use_case
            .execute(&auth_context(OTHER_BUYER))
            .await
            .expect("offers");

        assert_eq!(placed.len(), 1);
        assert_eq!(*placed[0].listing_status(), ListingStatus::Sold);
    }

    #[tokio::test]
    async fn a_phone_without_offers_reads_no_listings() {
        let repository = a_buyer_with_two_offers();
        let use_case = ListMyOffersUseCase::new(Arc::new(repository.clone()));

        let placed = use_case
            .execute(&auth_context(SELLER))
            .await
            .expect("offers");

        assert!(placed.is_empty());
        assert_eq!(repository.calls().len(), 1);
    }
}
