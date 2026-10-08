use std::sync::Arc;

use chrono::Utc;

use crate::{
    app::{AppError as GlobalAppError, AuthContext},
    features::alwa::{
        app::{AlwaRepository, AppError},
        domain::{Offer, OfferDraft},
    },
};

pub struct MakeOfferUseCase {
    repository: Arc<dyn AlwaRepository>,
}

impl MakeOfferUseCase {
    pub fn new(repository: Arc<dyn AlwaRepository>) -> Self {
        Self { repository }
    }

    /// The signed-in phone offers on a listing. An open offer the same phone
    /// already has on it is withdrawn in favour of this one.
    pub async fn execute(
        &self,
        auth_context: &AuthContext,
        listing_id: i32,
        draft: OfferDraft,
    ) -> Result<Offer, AppError> {
        let Some(listing) = self.repository.find_listing_by_id(listing_id).await? else {
            tracing::info!(listing_id, "offer refused: no such listing");

            return Err(GlobalAppError::NotFound.into());
        };

        let mut existing = self
            .repository
            .find_offers_by_listings(&[listing_id])
            .await?;

        let offer = listing
            .place_offer(
                auth_context.user().phone().clone(),
                draft,
                &mut existing,
                Utc::now(),
            )
            .inspect_err(|error| tracing::info!(listing_id, %error, "offer refused"))?;

        let placed = self.repository.place_offer(&offer).await?;

        tracing::info!(
            listing_id,
            offer_id = placed.id().unwrap_or_default(),
            price_iqd_per_kg = placed.price().value(),
            quantity_kg = placed.quantity().value(),
            "offer made"
        );

        Ok(placed)
    }
}

#[cfg(test)]
mod tests {
    use chrono::Duration;

    use super::*;
    use crate::features::alwa::{
        app::testing::{
            BUYER, FakeAlwaRepository, OTHER_BUYER, RepositoryCall, SELLER, a_listing,
            a_sold_listing, an_offer_draft, an_open_listing, an_open_offer, auth_context,
        },
        domain::{AlwaError, OfferStatus},
    };

    #[tokio::test]
    async fn a_buyer_makes_an_open_offer_under_their_own_phone() {
        let repository = FakeAlwaRepository::new().with_listing(an_open_listing(7));
        let use_case = MakeOfferUseCase::new(Arc::new(repository.clone()));

        let offer = use_case
            .execute(&auth_context(BUYER), 7, an_offer_draft(200, 950))
            .await
            .expect("offer");

        assert!(offer.is_open());
        assert!(offer.id().is_some());
        assert_eq!(offer.buyer_phone().as_str(), BUYER);
        assert!(repository.calls().contains(&RepositoryCall::PlaceOffer {
            listing_id: 7,
            buyer: BUYER.to_string(),
        }));
    }

    #[tokio::test]
    async fn a_second_offer_replaces_the_buyers_first_and_leaves_the_others() {
        let listing = an_open_listing(7);
        let repository = FakeAlwaRepository::new()
            .with_offer(an_open_offer(1, &listing, BUYER, 900))
            .with_offer(an_open_offer(2, &listing, OTHER_BUYER, 920))
            .with_listing(listing);
        let use_case = MakeOfferUseCase::new(Arc::new(repository.clone()));

        let newer = use_case
            .execute(&auth_context(BUYER), 7, an_offer_draft(500, 960))
            .await
            .expect("offer");

        let status_of = |id: Option<i32>| {
            repository
                .stored_offers()
                .into_iter()
                .find(|offer| *offer.id() == id)
                .map(|offer| *offer.status())
        };

        assert_eq!(status_of(Some(1)), Some(OfferStatus::Withdrawn));
        assert_eq!(status_of(Some(2)), Some(OfferStatus::Open));
        assert_eq!(status_of(*newer.id()), Some(OfferStatus::Open));
        assert_eq!(
            repository
                .stored_offers()
                .iter()
                .filter(|offer| offer.is_open() && offer.buyer_phone().as_str() == BUYER)
                .count(),
            1,
            "a buyer has one open offer on a listing at most"
        );
    }

    #[tokio::test]
    async fn the_seller_cannot_offer_on_their_own_listing() {
        let repository = FakeAlwaRepository::new().with_listing(an_open_listing(7));
        let use_case = MakeOfferUseCase::new(Arc::new(repository.clone()));

        let result = use_case
            .execute(&auth_context(SELLER), 7, an_offer_draft(200, 950))
            .await;

        assert!(matches!(result, Err(AppError::Alwa(AlwaError::OwnListing))));
        assert!(!repository.wrote());
    }

    #[tokio::test]
    async fn no_offer_is_written_on_a_listing_that_is_sold_or_past_its_closing_time() {
        let (sold, _) = a_sold_listing(7);
        let repository = FakeAlwaRepository::new()
            .with_listing(sold)
            .with_listing(a_listing(8, Utc::now() - Duration::days(5)));
        let use_case = MakeOfferUseCase::new(Arc::new(repository.clone()));

        for id in [7, 8] {
            assert!(matches!(
                use_case
                    .execute(&auth_context(OTHER_BUYER), id, an_offer_draft(200, 950))
                    .await,
                Err(AppError::Alwa(AlwaError::ListingNotOpen))
            ));
        }

        assert!(!repository.wrote());
    }

    #[tokio::test]
    async fn an_offer_for_more_than_is_on_sale_is_not_written() {
        let repository = FakeAlwaRepository::new().with_listing(an_open_listing(7));
        let use_case = MakeOfferUseCase::new(Arc::new(repository.clone()));

        let result = use_case
            .execute(&auth_context(BUYER), 7, an_offer_draft(501, 950))
            .await;

        assert!(matches!(
            result,
            Err(AppError::Alwa(AlwaError::OfferTooLarge(500)))
        ));
        assert!(!repository.wrote());
    }

    #[tokio::test]
    async fn an_unknown_listing_is_not_found() {
        let repository = FakeAlwaRepository::new();
        let use_case = MakeOfferUseCase::new(Arc::new(repository.clone()));

        let result = use_case
            .execute(&auth_context(BUYER), 7, an_offer_draft(200, 950))
            .await;

        assert!(matches!(
            result,
            Err(AppError::GlobalAppError(GlobalAppError::NotFound))
        ));
        assert_eq!(
            repository.calls(),
            vec![RepositoryCall::FindListingById { id: 7 }]
        );
    }
}
