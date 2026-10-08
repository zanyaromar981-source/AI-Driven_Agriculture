use std::sync::Arc;

use chrono::Utc;

use crate::{
    app::{AppError as GlobalAppError, AuthContext},
    features::alwa::{
        app::{AlwaRepository, AppError, listing_cards::assemble_cards},
        domain::{Listing, ListingCard, ListingDraft, MarketSlug},
    },
};

pub struct PostListingInput {
    pub market: MarketSlug,
    pub draft: ListingDraft,
}

pub struct PostListingUseCase {
    repository: Arc<dyn AlwaRepository>,
    max_open_listings_per_seller: u64,
}

impl PostListingUseCase {
    pub fn new(repository: Arc<dyn AlwaRepository>, max_open_listings_per_seller: u64) -> Self {
        Self {
            repository,
            max_open_listings_per_seller,
        }
    }

    pub async fn execute(
        &self,
        auth_context: &AuthContext,
        input: PostListingInput,
    ) -> Result<ListingCard, AppError> {
        let seller = auth_context.user().phone();
        let now = Utc::now();

        let Some(market) = self.repository.find_market_by_slug(&input.market).await? else {
            tracing::info!(
                market = input.market.as_str(),
                "listing refused: no such alwa"
            );

            return Err(GlobalAppError::NotFound.into());
        };

        let open = self
            .repository
            .count_open_listings_by_seller(seller, now)
            .await?;

        if open >= self.max_open_listings_per_seller {
            tracing::info!(
                open,
                max = self.max_open_listings_per_seller,
                "listing refused: open listing quota reached"
            );

            return Err(AppError::MaxOpenListingsReached(
                self.max_open_listings_per_seller,
            ));
        }

        let listing = Listing::new(seller.clone(), &market, input.draft, now)?;
        let posted = self.repository.create_listing(&listing).await?;

        tracing::info!(
            listing_id = posted.id().unwrap_or_default(),
            market = market.slug().as_str(),
            crop = String::from(*posted.crop()),
            quantity_kg = posted.quantity().value(),
            open_after = open + 1,
            "listing posted"
        );

        let cards = assemble_cards(self.repository.as_ref(), vec![posted], now).await?;

        cards.into_iter().next().ok_or_else(|| {
            GlobalAppError::MissingValue("The posted listing has no card".to_string()).into()
        })
    }
}

#[cfg(test)]
mod tests {
    use chrono::Duration;

    use super::*;
    use crate::features::alwa::{
        app::testing::{
            FakeAlwaRepository, MARKET, RepositoryCall, SELLER, a_listing, a_listing_draft,
            an_open_listing, auth_context, market_slug,
        },
        domain::{AlwaError, ListingStatus},
    };

    const MAX: u64 = 3;

    fn input() -> PostListingInput {
        PostListingInput {
            market: market_slug(MARKET),
            draft: a_listing_draft(Utc::now() + Duration::days(2)),
        }
    }

    fn holding(open: u64) -> FakeAlwaRepository {
        (0..open).fold(FakeAlwaRepository::new(), |repository, id| {
            repository.with_listing(an_open_listing(id as i32 + 1))
        })
    }

    #[tokio::test]
    async fn posts_an_open_listing_for_the_signed_in_phone() {
        let repository = holding(MAX - 1);
        let use_case = PostListingUseCase::new(Arc::new(repository.clone()), MAX);

        let card = use_case
            .execute(&auth_context(SELLER), input())
            .await
            .expect("card");

        assert_eq!(card.listing().seller_phone().as_str(), SELLER);
        assert_eq!(*card.status(), ListingStatus::Open);
        assert!(card.listing().id().is_some());
        assert!(card.offers().is_empty());
        assert!(repository.calls().contains(&RepositoryCall::CreateListing));
    }

    #[tokio::test]
    async fn rejects_once_the_open_listing_quota_is_reached() {
        let repository = holding(MAX);
        let use_case = PostListingUseCase::new(Arc::new(repository.clone()), MAX);

        let result = use_case.execute(&auth_context(SELLER), input()).await;

        assert!(matches!(result, Err(AppError::MaxOpenListingsReached(MAX))));
        assert!(
            !repository.wrote(),
            "a rejected listing must not be written"
        );
        assert!(
            repository
                .calls()
                .contains(&RepositoryCall::CountOpenListingsBySeller {
                    seller: SELLER.to_string()
                })
        );
    }

    #[tokio::test]
    async fn listings_that_are_no_longer_open_do_not_count_against_the_quota() {
        let repository =
            holding(MAX - 1).with_listing(a_listing(50, Utc::now() - Duration::days(5)));
        let use_case = PostListingUseCase::new(Arc::new(repository.clone()), MAX);

        assert!(
            use_case
                .execute(&auth_context(SELLER), input())
                .await
                .is_ok(),
            "a listing past its closing time is closed, so it frees its place"
        );
    }

    #[tokio::test]
    async fn a_closing_time_in_the_past_is_not_written() {
        let repository = FakeAlwaRepository::new();
        let use_case = PostListingUseCase::new(Arc::new(repository.clone()), MAX);

        let result = use_case
            .execute(
                &auth_context(SELLER),
                PostListingInput {
                    market: market_slug(MARKET),
                    draft: a_listing_draft(Utc::now() - Duration::minutes(1)),
                },
            )
            .await;

        assert!(matches!(
            result,
            Err(AppError::Alwa(AlwaError::BadClosingTime(_)))
        ));
        assert!(!repository.wrote());
    }

    #[tokio::test]
    async fn a_listing_at_an_unknown_alwa_is_not_found() {
        let repository = FakeAlwaRepository::new();
        let use_case = PostListingUseCase::new(Arc::new(repository.clone()), MAX);

        let result = use_case
            .execute(
                &auth_context(SELLER),
                PostListingInput {
                    market: market_slug("baghdad"),
                    ..input()
                },
            )
            .await;

        assert!(matches!(
            result,
            Err(AppError::GlobalAppError(GlobalAppError::NotFound))
        ));
        assert!(!repository.wrote());
    }
}
