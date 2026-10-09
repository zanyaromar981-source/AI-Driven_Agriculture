use std::sync::Arc;

use chrono::Utc;

use crate::{
    app::{AppError as GlobalAppError, AuthContext},
    features::alwa::{
        app::{AlwaRepository, AppError, listing_cards::assemble_cards},
        domain::{AlwaError, Listing, ListingCard},
    },
};

pub struct MarkListingSoldUseCase {
    repository: Arc<dyn AlwaRepository>,
}

impl MarkListingSoldUseCase {
    pub fn new(repository: Arc<dyn AlwaRepository>) -> Self {
        Self { repository }
    }

    /// The seller says the crop is sold, to a buyer who called them. No
    /// offer is needed; the offers still open are declined. Returns the
    /// listing as it stands afterwards.
    pub async fn execute(
        &self,
        auth_context: &AuthContext,
        id: i32,
    ) -> Result<ListingCard, AppError> {
        let seller = auth_context.user().phone();

        let Some(mut listing) = self.repository.find_listing_by_id(id).await? else {
            tracing::info!(listing_id = id, "sold refused: no such listing");

            return Err(GlobalAppError::NotFound.into());
        };

        let now = Utc::now();

        // The app repeats a "sold" whose answer was lost. The listing being
        // sold is what it asked for, so the repeat gets the same answer.
        if listing.was_marked_sold_by(seller) {
            tracing::info!(listing_id = id, "listing already sold: nothing to do");

            return first_card(self.repository.as_ref(), listing, now).await;
        }

        listing
            .mark_sold(seller, now)
            .inspect_err(|error| tracing::info!(listing_id = id, %error, "sold refused"))?;

        match self.repository.sell_listing(&listing).await {
            Ok(()) => {}
            // Two copies of one "sold" can both pass the read above; the
            // database lets one in, and the other finds the listing as its
            // twin left it. Anything else that closed it first is refused.
            Err(AppError::Alwa(AlwaError::ListingNotOpen)) => {
                let stored = self.repository.find_listing_by_id(id).await?;

                return match stored {
                    Some(stored) if stored.was_marked_sold_by(seller) => {
                        first_card(self.repository.as_ref(), stored, now).await
                    }
                    _ => Err(AlwaError::ListingNotOpen.into()),
                };
            }
            Err(error) => return Err(error),
        }

        tracing::info!(listing_id = id, "listing marked sold by its seller");

        first_card(self.repository.as_ref(), listing, now).await
    }
}

async fn first_card(
    repository: &dyn AlwaRepository,
    listing: Listing,
    now: chrono::DateTime<Utc>,
) -> Result<ListingCard, AppError> {
    let cards = assemble_cards(repository, vec![listing], now).await?;

    cards.into_iter().next().ok_or_else(|| {
        GlobalAppError::MissingValue("The sold listing has no card".to_string()).into()
    })
}

#[cfg(test)]
mod tests {
    use chrono::Duration;

    use super::*;
    use crate::features::alwa::{
        app::testing::{
            BUYER, FakeAlwaRepository, OTHER_BUYER, RepositoryCall, SELLER, a_listing,
            a_sold_listing, an_open_listing, an_open_offer, auth_context, phone,
        },
        domain::{ListingStatus, OfferStatus},
    };

    fn use_case(repository: &FakeAlwaRepository) -> MarkListingSoldUseCase {
        MarkListingSoldUseCase::new(Arc::new(repository.clone()))
    }

    #[tokio::test]
    async fn the_seller_marks_their_open_listing_sold_without_any_offer() {
        let repository = FakeAlwaRepository::new().with_listing(an_open_listing(7));

        let card = use_case(&repository)
            .execute(&auth_context(SELLER), 7)
            .await
            .expect("card");

        assert_eq!(*card.status(), ListingStatus::Sold);
        assert!(card.listing().sold_at().is_some());
        assert_eq!(
            repository
                .stored_listing(7)
                .map(|listing| *listing.status()),
            Some(ListingStatus::Sold)
        );
        assert!(
            repository
                .calls()
                .contains(&RepositoryCall::SellListing { id: 7 })
        );
    }

    #[tokio::test]
    async fn the_offers_still_open_are_declined_and_the_answer_shows_it() {
        let listing = an_open_listing(7);
        let repository = FakeAlwaRepository::new()
            .with_offer(an_open_offer(1, &listing, BUYER, 950))
            .with_offer(an_open_offer(2, &listing, OTHER_BUYER, 900))
            .with_listing(listing);

        let card = use_case(&repository)
            .execute(&auth_context(SELLER), 7)
            .await
            .expect("card");

        assert!(
            card.offers()
                .iter()
                .all(|offer| *offer.status() == OfferStatus::Declined),
            "nobody's offer became the deal"
        );
        assert_eq!(card.open_offers(), 0);
        assert!(
            repository
                .stored_offers()
                .iter()
                .all(|offer| *offer.status() == OfferStatus::Declined)
        );
    }

    #[tokio::test]
    async fn a_repeat_gets_the_same_answer_and_writes_nothing_more() {
        let repository = FakeAlwaRepository::new().with_listing(an_open_listing(7));
        let use_case = use_case(&repository);

        let first = use_case
            .execute(&auth_context(SELLER), 7)
            .await
            .expect("first");
        let second = use_case
            .execute(&auth_context(SELLER), 7)
            .await
            .expect("second");

        assert_eq!(*second.status(), ListingStatus::Sold);
        assert_eq!(second.listing().sold_at(), first.listing().sold_at());
        assert_eq!(
            repository
                .calls()
                .iter()
                .filter(|call| **call == RepositoryCall::SellListing { id: 7 })
                .count(),
            1
        );
    }

    #[tokio::test]
    async fn a_listing_already_sold_through_an_offer_answers_the_same_success() {
        let (listing, offers) = a_sold_listing(7);
        let repository = offers
            .into_iter()
            .fold(FakeAlwaRepository::new(), FakeAlwaRepository::with_offer)
            .with_listing(listing);

        let card = use_case(&repository)
            .execute(&auth_context(SELLER), 7)
            .await
            .expect("card");

        assert_eq!(*card.status(), ListingStatus::Sold);
        assert!(!repository.wrote());
    }

    #[tokio::test]
    async fn another_farmers_listing_is_not_found_and_stays_open() {
        let repository = FakeAlwaRepository::new().with_listing(an_open_listing(7));

        let result = use_case(&repository).execute(&auth_context(BUYER), 7).await;

        assert!(matches!(
            result,
            Err(AppError::Alwa(AlwaError::NotTheSeller))
        ));
        assert!(!repository.wrote());
    }

    #[tokio::test]
    async fn another_farmers_sold_listing_is_not_a_repeat_either() {
        let (listing, _) = a_sold_listing(7);
        let repository = FakeAlwaRepository::new().with_listing(listing);

        let result = use_case(&repository).execute(&auth_context(BUYER), 7).await;

        assert!(matches!(
            result,
            Err(AppError::Alwa(AlwaError::NotTheSeller))
        ));
    }

    #[tokio::test]
    async fn a_listing_that_does_not_exist_is_not_found() {
        let repository = FakeAlwaRepository::new();

        let result = use_case(&repository)
            .execute(&auth_context(SELLER), 7)
            .await;

        assert!(matches!(
            result,
            Err(AppError::GlobalAppError(GlobalAppError::NotFound))
        ));
    }

    #[tokio::test]
    async fn a_cancelled_or_expired_listing_is_not_open() {
        let mut cancelled = an_open_listing(7);
        cancelled
            .cancel(&phone(SELLER), Utc::now())
            .expect("cancel");
        let expired = a_listing(8, Utc::now() - Duration::days(5));
        let repository = FakeAlwaRepository::new()
            .with_listing(cancelled)
            .with_listing(expired);

        for id in [7, 8] {
            let result = use_case(&repository)
                .execute(&auth_context(SELLER), id)
                .await;

            assert!(
                matches!(result, Err(AppError::Alwa(AlwaError::ListingNotOpen))),
                "listing {id}"
            );
        }

        assert!(!repository.wrote());
    }

    #[tokio::test]
    async fn when_a_twin_request_sold_it_first_this_one_answers_the_same_success() {
        let mut twin = an_open_listing(7);
        twin.mark_sold(&phone(SELLER), Utc::now()).expect("sold");
        let repository = FakeAlwaRepository::new()
            .with_listing(an_open_listing(7))
            .with_rival_write(twin);

        let card = use_case(&repository)
            .execute(&auth_context(SELLER), 7)
            .await
            .expect("card");

        assert_eq!(*card.status(), ListingStatus::Sold);
    }

    #[tokio::test]
    async fn when_a_cancel_got_there_first_the_listing_is_not_open() {
        let mut cancelled = an_open_listing(7);
        cancelled
            .cancel(&phone(SELLER), Utc::now())
            .expect("cancel");
        let repository = FakeAlwaRepository::new()
            .with_listing(an_open_listing(7))
            .with_rival_write(cancelled);

        let result = use_case(&repository)
            .execute(&auth_context(SELLER), 7)
            .await;

        assert!(matches!(
            result,
            Err(AppError::Alwa(AlwaError::ListingNotOpen))
        ));
        assert_eq!(
            repository
                .stored_listing(7)
                .map(|listing| *listing.status()),
            Some(ListingStatus::Cancelled)
        );
    }
}
