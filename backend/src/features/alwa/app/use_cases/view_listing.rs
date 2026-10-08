use std::sync::Arc;

use chrono::Utc;

use crate::{
    app::AppError as GlobalAppError,
    features::alwa::{
        app::{AlwaRepository, AppError, listing_cards::assemble_cards},
        domain::ListingCard,
    },
};

pub struct ViewListingUseCase {
    repository: Arc<dyn AlwaRepository>,
}

impl ViewListingUseCase {
    pub fn new(repository: Arc<dyn AlwaRepository>) -> Self {
        Self { repository }
    }

    /// Anyone may look at a listing and its offers, as on the market floor.
    pub async fn execute(&self, id: i32) -> Result<ListingCard, AppError> {
        let Some(listing) = self.repository.find_listing_by_id(id).await? else {
            tracing::info!(listing_id = id, "view refused: no such listing");

            return Err(GlobalAppError::NotFound.into());
        };

        let cards = assemble_cards(self.repository.as_ref(), vec![listing], Utc::now()).await?;

        cards
            .into_iter()
            .next()
            .ok_or_else(|| GlobalAppError::NotFound.into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::features::alwa::app::testing::{
        BUYER, FakeAlwaRepository, OTHER_BUYER, RepositoryCall, an_open_listing, an_open_offer,
    };

    #[tokio::test]
    async fn returns_the_listing_with_its_offers_highest_first() {
        let listing = an_open_listing(7);
        let repository = FakeAlwaRepository::new()
            .with_offer(an_open_offer(1, &listing, BUYER, 900))
            .with_offer(an_open_offer(2, &listing, OTHER_BUYER, 950))
            .with_listing(listing);
        let use_case = ViewListingUseCase::new(Arc::new(repository.clone()));

        let card = use_case.execute(7).await.expect("card");

        assert_eq!(
            card.offers()
                .iter()
                .map(|offer| offer.price().value())
                .collect::<Vec<_>>(),
            vec![950, 900]
        );
        assert!(
            repository
                .calls()
                .contains(&RepositoryCall::FindOffersByListings {
                    listing_ids: vec![7]
                })
        );
    }

    #[tokio::test]
    async fn an_unknown_listing_is_not_found() {
        let use_case = ViewListingUseCase::new(Arc::new(FakeAlwaRepository::new()));

        assert!(matches!(
            use_case.execute(7).await,
            Err(AppError::GlobalAppError(GlobalAppError::NotFound))
        ));
    }
}
