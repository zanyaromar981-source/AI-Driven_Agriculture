use std::sync::Arc;

use chrono::Utc;

use crate::{
    app::AppError as GlobalAppError,
    features::alwa::{
        app::{AlwaRepository, AppError, listing_cards::assemble_cards},
        domain::{AlwaError, Listing, ListingCard, ListingStatus, Note},
    },
};

pub struct ModerateListingInput {
    pub id: i32,
    /// What the listing is to become. Only `Closed` is ever allowed.
    pub status: ListingStatus,
    pub note: Option<Note>,
}

pub struct ModerateListingUseCase {
    repository: Arc<dyn AlwaRepository>,
}

impl ModerateListingUseCase {
    pub fn new(repository: Arc<dyn AlwaRepository>) -> Self {
        Self { repository }
    }

    /// A staff member closes an open listing; its open offers are declined
    /// with it.
    pub async fn execute(
        &self,
        staff_id: i32,
        input: ModerateListingInput,
    ) -> Result<ListingCard, AppError> {
        let id = input.id;

        let Some(mut listing) = self.repository.find_listing_by_id(id).await? else {
            tracing::info!(
                staff_id,
                listing_id = id,
                "moderation refused: no such listing"
            );

            return Err(GlobalAppError::NotFound.into());
        };

        // The dashboard repeats a close whose answer was lost. The listing
        // being closed with that note is what it asked for, so the repeat
        // succeeds.
        if input.status == ListingStatus::Closed
            && listing.was_closed_by_staff_with(input.note.as_ref())
        {
            tracing::info!(
                staff_id,
                listing_id = id,
                "listing already closed: nothing to do"
            );

            return self.card(listing).await;
        }

        listing
            .moderate(input.status, staff_id, input.note.clone(), Utc::now())
            .inspect_err(|error| {
                tracing::info!(staff_id, listing_id = id, %error, "moderation refused");
            })?;

        match self.repository.close_listing(&listing).await {
            Ok(()) => {
                tracing::info!(staff_id, listing_id = id, "listing closed by staff");
            }
            // Someone got to the listing between the read and the write. If
            // that was this same close sent twice, both get the same answer.
            Err(AppError::Alwa(AlwaError::ListingNotOpen)) => {
                listing = self
                    .repository
                    .find_listing_by_id(id)
                    .await?
                    .filter(|stored| stored.was_closed_by_staff_with(input.note.as_ref()))
                    .ok_or(AlwaError::ListingNotOpen)
                    .inspect_err(|error| {
                        tracing::info!(staff_id, listing_id = id, %error, "moderation refused");
                    })?;
            }
            Err(error) => return Err(error),
        }

        self.card(listing).await
    }

    async fn card(&self, listing: Listing) -> Result<ListingCard, AppError> {
        assemble_cards(self.repository.as_ref(), vec![listing], Utc::now())
            .await?
            .into_iter()
            .next()
            .ok_or_else(|| GlobalAppError::NotFound.into())
    }
}

#[cfg(test)]
mod tests {
    use chrono::Duration;

    use super::*;
    use crate::features::alwa::{
        app::testing::{
            BUYER, FakeAlwaRepository, OTHER_BUYER, RepositoryCall, SELLER, a_listing,
            a_sold_listing, an_open_listing, an_open_offer, phone,
        },
        domain::{Moderation, OfferStatus},
    };

    fn note(text: &str) -> Option<Note> {
        Some(Note::new(text.to_string()).expect("note"))
    }

    fn close(note: Option<Note>) -> ModerateListingInput {
        ModerateListingInput {
            id: 7,
            status: ListingStatus::Closed,
            note,
        }
    }

    fn closes(repository: &FakeAlwaRepository) -> usize {
        repository
            .calls()
            .iter()
            .filter(|call| matches!(call, RepositoryCall::CloseListing { .. }))
            .count()
    }

    #[tokio::test]
    async fn closing_declines_the_open_offers_and_records_who_closed() {
        let listing = an_open_listing(7);
        let repository = FakeAlwaRepository::new()
            .with_offer(an_open_offer(1, &listing, BUYER, 900))
            .with_offer(an_open_offer(2, &listing, OTHER_BUYER, 950))
            .with_listing(listing);
        let use_case = ModerateListingUseCase::new(Arc::new(repository.clone()));

        let card = use_case
            .execute(9, close(note("spam")))
            .await
            .expect("close");

        assert_eq!(*card.status(), ListingStatus::Closed);
        assert_eq!(card.open_offers(), 0);
        assert_eq!(
            card.listing().moderation(),
            &Some(Moderation::new(9, note("spam")))
        );
        assert!(
            repository
                .stored_offers()
                .iter()
                .all(|offer| *offer.status() == OfferStatus::Declined)
        );
        assert_eq!(
            repository.calls()[..2],
            [
                RepositoryCall::FindListingById { id: 7 },
                RepositoryCall::CloseListing { id: 7 },
            ]
        );
    }

    #[tokio::test]
    async fn the_same_close_again_succeeds_and_writes_nothing_more() {
        let repository = FakeAlwaRepository::new().with_listing(an_open_listing(7));
        let use_case = ModerateListingUseCase::new(Arc::new(repository.clone()));

        use_case
            .execute(9, close(note("spam")))
            .await
            .expect("first");
        let repeat = use_case
            .execute(9, close(note("spam")))
            .await
            .expect("repeat");

        assert_eq!(*repeat.status(), ListingStatus::Closed);
        assert_eq!(closes(&repository), 1, "the repeat must not write again");
    }

    #[tokio::test]
    async fn a_close_with_another_note_is_a_different_request_and_is_refused() {
        let repository = FakeAlwaRepository::new().with_listing(an_open_listing(7));
        let use_case = ModerateListingUseCase::new(Arc::new(repository.clone()));

        use_case
            .execute(9, close(note("spam")))
            .await
            .expect("first");

        for other in [note("fraud"), None] {
            assert!(matches!(
                use_case.execute(9, close(other)).await,
                Err(AppError::Alwa(AlwaError::ListingNotOpen))
            ));
        }

        assert_eq!(
            repository
                .stored_listing(7)
                .and_then(|listing| listing.moderation().clone()),
            Some(Moderation::new(9, note("spam"))),
            "the first close stands"
        );
    }

    #[tokio::test]
    async fn staff_cannot_sell_reopen_or_cancel() {
        let repository = FakeAlwaRepository::new().with_listing(an_open_listing(7));
        let use_case = ModerateListingUseCase::new(Arc::new(repository.clone()));

        for status in [
            ListingStatus::Sold,
            ListingStatus::Open,
            ListingStatus::Cancelled,
        ] {
            let result = use_case
                .execute(
                    9,
                    ModerateListingInput {
                        id: 7,
                        status,
                        note: None,
                    },
                )
                .await;

            assert!(matches!(
                result,
                Err(AppError::Alwa(AlwaError::StaffMayOnlyClose))
            ));
        }

        assert!(!repository.wrote());
    }

    #[tokio::test]
    async fn a_listing_that_is_sold_cancelled_or_past_its_time_is_not_closed() {
        let (sold, offers) = a_sold_listing(7);
        let mut cancelled = an_open_listing(7);
        cancelled
            .cancel(&phone(SELLER), Utc::now())
            .expect("cancel");
        let expired = a_listing(7, Utc::now() - Duration::days(5));

        for listing in [sold, cancelled, expired] {
            let repository = offers
                .iter()
                .cloned()
                .fold(FakeAlwaRepository::new(), FakeAlwaRepository::with_offer)
                .with_listing(listing);
            let use_case = ModerateListingUseCase::new(Arc::new(repository.clone()));

            assert!(matches!(
                use_case.execute(9, close(None)).await,
                Err(AppError::Alwa(AlwaError::ListingNotOpen))
            ));
            assert!(!repository.wrote());
            assert!(
                repository
                    .stored_offers()
                    .iter()
                    .any(|offer| offer.is_accepted()),
                "the deal's offer must be left alone"
            );
        }
    }

    #[tokio::test]
    async fn a_deal_made_between_the_read_and_the_write_wins() {
        let (sold, _) = a_sold_listing(7);
        let repository = FakeAlwaRepository::new()
            .with_listing(an_open_listing(7))
            .with_rival_write(sold);
        let use_case = ModerateListingUseCase::new(Arc::new(repository.clone()));

        let result = use_case.execute(9, close(None)).await;

        assert!(matches!(
            result,
            Err(AppError::Alwa(AlwaError::ListingNotOpen))
        ));
        assert_eq!(
            repository
                .stored_listing(7)
                .map(|listing| *listing.status()),
            Some(ListingStatus::Sold)
        );
    }

    #[tokio::test]
    async fn the_same_close_sent_twice_at_once_answers_both_alike() {
        let mut already = an_open_listing(7);
        already
            .moderate(ListingStatus::Closed, 9, note("spam"), Utc::now())
            .expect("close");
        let repository = FakeAlwaRepository::new()
            .with_listing(an_open_listing(7))
            .with_rival_write(already);
        let use_case = ModerateListingUseCase::new(Arc::new(repository.clone()));

        let card = use_case
            .execute(9, close(note("spam")))
            .await
            .expect("close");

        assert_eq!(*card.status(), ListingStatus::Closed);
    }

    #[tokio::test]
    async fn an_unknown_listing_is_not_found() {
        let use_case = ModerateListingUseCase::new(Arc::new(FakeAlwaRepository::new()));

        assert!(matches!(
            use_case.execute(9, close(None)).await,
            Err(AppError::GlobalAppError(GlobalAppError::NotFound))
        ));
    }
}
