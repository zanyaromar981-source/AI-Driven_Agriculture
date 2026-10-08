use std::sync::Arc;

use chrono::Utc;

use crate::{
    app::{AppError as GlobalAppError, AuthContext},
    features::alwa::{
        app::{AlwaRepository, AppError, listing_cards::assemble_cards},
        domain::ListingCard,
    },
};

pub struct AcceptOfferUseCase {
    repository: Arc<dyn AlwaRepository>,
}

impl AcceptOfferUseCase {
    pub fn new(repository: Arc<dyn AlwaRepository>) -> Self {
        Self { repository }
    }

    /// The seller takes one offer and the deal is made. Returns the listing
    /// as it stands afterwards.
    pub async fn execute(
        &self,
        auth_context: &AuthContext,
        listing_id: i32,
        offer_id: i32,
    ) -> Result<ListingCard, AppError> {
        let Some(mut listing) = self.repository.find_listing_by_id(listing_id).await? else {
            tracing::info!(listing_id, "accept refused: no such listing");

            return Err(GlobalAppError::NotFound.into());
        };

        let mut offers = self
            .repository
            .find_offers_by_listings(&[listing_id])
            .await?;
        let now = Utc::now();

        listing
            .accept(auth_context.user().phone(), offer_id, &mut offers, now)
            .inspect_err(|error| tracing::info!(listing_id, offer_id, %error, "accept refused"))?;

        let Some(accepted) = offers.iter().find(|offer| *offer.id() == Some(offer_id)) else {
            return Err(
                GlobalAppError::MissingValue("The accepted offer is gone".to_string()).into(),
            );
        };

        self.repository.accept_offer(&listing, accepted).await?;

        tracing::info!(
            listing_id,
            offer_id,
            sold_price_iqd_per_kg = accepted.price().value(),
            quantity_kg = accepted.quantity().value(),
            "offer accepted: deal made"
        );

        // Read back rather than trust the copies in memory: the answer shows
        // what the transaction really left behind.
        let cards = assemble_cards(self.repository.as_ref(), vec![listing], now).await?;

        cards.into_iter().next().ok_or_else(|| {
            GlobalAppError::MissingValue("The sold listing has no card".to_string()).into()
        })
    }
}

#[cfg(test)]
mod tests {
    use chrono::Duration;

    use super::*;
    use crate::features::alwa::{
        app::testing::{
            BUYER, FakeAlwaRepository, OTHER_BUYER, RepositoryCall, SELLER, a_listing,
            a_sold_listing, an_open_listing, an_open_offer, auth_context,
        },
        domain::{AlwaError, ListingStatus, OfferStatus},
    };

    fn a_listing_with_two_offers() -> FakeAlwaRepository {
        let listing = an_open_listing(7);

        FakeAlwaRepository::new()
            .with_offer(an_open_offer(1, &listing, BUYER, 950))
            .with_offer(an_open_offer(2, &listing, OTHER_BUYER, 900))
            .with_listing(listing)
    }

    #[tokio::test]
    async fn accepting_sells_the_listing_and_declines_the_other_offer() {
        let repository = a_listing_with_two_offers();
        let use_case = AcceptOfferUseCase::new(Arc::new(repository.clone()));

        let card = use_case
            .execute(&auth_context(SELLER), 7, 1)
            .await
            .expect("card");

        assert_eq!(*card.status(), ListingStatus::Sold);
        assert_eq!(
            card.offers()
                .iter()
                .map(|offer| (offer.id().unwrap_or_default(), *offer.status()))
                .collect::<Vec<_>>(),
            vec![(1, OfferStatus::Accepted), (2, OfferStatus::Declined)]
        );
        assert_eq!(
            repository
                .stored_listing(7)
                .map(|listing| *listing.status()),
            Some(ListingStatus::Sold)
        );
        assert!(repository.calls().contains(&RepositoryCall::AcceptOffer {
            listing_id: 7,
            offer_id: 1
        }));
    }

    #[tokio::test]
    async fn the_deal_is_one_write_not_three() {
        let repository = a_listing_with_two_offers();
        let use_case = AcceptOfferUseCase::new(Arc::new(repository.clone()));

        use_case
            .execute(&auth_context(SELLER), 7, 1)
            .await
            .expect("card");

        assert_eq!(
            repository
                .calls()
                .iter()
                .filter(|call| matches!(
                    call,
                    RepositoryCall::AcceptOffer { .. }
                        | RepositoryCall::CancelListing { .. }
                        | RepositoryCall::PlaceOffer { .. }
                ))
                .count(),
            1,
            "the listing and its offers must change together or not at all"
        );
    }

    #[tokio::test]
    async fn a_buyer_cannot_accept_their_own_offer() {
        let repository = a_listing_with_two_offers();
        let use_case = AcceptOfferUseCase::new(Arc::new(repository.clone()));

        let result = use_case.execute(&auth_context(BUYER), 7, 1).await;

        assert!(matches!(
            result,
            Err(AppError::Alwa(AlwaError::NotTheSeller))
        ));
        assert!(!repository.wrote());
    }

    #[tokio::test]
    async fn an_offer_from_another_listing_is_not_accepted() {
        let other = an_open_listing(8);
        let repository = a_listing_with_two_offers()
            .with_offer(an_open_offer(3, &other, BUYER, 990))
            .with_listing(other);
        let use_case = AcceptOfferUseCase::new(Arc::new(repository.clone()));

        let result = use_case.execute(&auth_context(SELLER), 7, 3).await;

        assert!(matches!(
            result,
            Err(AppError::Alwa(AlwaError::OfferNotOnListing))
        ));
        assert!(!repository.wrote());
    }

    #[tokio::test]
    async fn a_sold_listing_cannot_accept_a_second_offer() {
        let (sold, offers) = a_sold_listing(7);
        let repository = offers.into_iter().fold(
            FakeAlwaRepository::new().with_listing(sold),
            FakeAlwaRepository::with_offer,
        );
        let use_case = AcceptOfferUseCase::new(Arc::new(repository.clone()));

        let result = use_case.execute(&auth_context(SELLER), 7, 2).await;

        assert!(matches!(
            result,
            Err(AppError::Alwa(AlwaError::ListingNotOpen))
        ));
        assert!(!repository.wrote());
    }

    #[tokio::test]
    async fn a_listing_past_its_closing_time_cannot_accept() {
        let expired = a_listing(7, Utc::now() - Duration::days(5));
        let repository = FakeAlwaRepository::new()
            .with_offer(an_open_offer(1, &expired, BUYER, 950))
            .with_listing(expired);
        let use_case = AcceptOfferUseCase::new(Arc::new(repository.clone()));

        let result = use_case.execute(&auth_context(SELLER), 7, 1).await;

        assert!(matches!(
            result,
            Err(AppError::Alwa(AlwaError::ListingNotOpen))
        ));
        assert!(!repository.wrote());
    }

    #[tokio::test]
    async fn a_withdrawn_offer_cannot_be_accepted() {
        let listing = an_open_listing(7);
        let mut offers = [an_open_offer(1, &listing, BUYER, 900)];

        // The buyer replaced offer 1, which withdrew it.
        listing
            .place_offer(
                crate::features::alwa::app::testing::phone(BUYER),
                crate::features::alwa::app::testing::an_offer_draft(500, 950),
                &mut offers,
                Utc::now(),
            )
            .expect("newer offer");

        let [withdrawn] = offers;
        let repository = FakeAlwaRepository::new()
            .with_offer(withdrawn)
            .with_listing(listing);
        let use_case = AcceptOfferUseCase::new(Arc::new(repository.clone()));

        let result = use_case.execute(&auth_context(SELLER), 7, 1).await;

        assert!(matches!(
            result,
            Err(AppError::Alwa(AlwaError::OfferNotOpen))
        ));
        assert!(!repository.wrote());
    }

    #[tokio::test]
    async fn an_unknown_listing_is_not_found() {
        let use_case = AcceptOfferUseCase::new(Arc::new(FakeAlwaRepository::new()));

        assert!(matches!(
            use_case.execute(&auth_context(SELLER), 7, 1).await,
            Err(AppError::GlobalAppError(GlobalAppError::NotFound))
        ));
    }
}
