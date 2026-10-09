use std::sync::Arc;

use chrono::Utc;

use crate::{
    app::{AppError as GlobalAppError, Pagination},
    features::alwa::{
        app::{AlwaRepository, AppError, ListingFilter, listing_cards::assemble_cards},
        domain::{Crop, ListingCard, ListingStatus, MarketSlug},
    },
};

pub struct BrowseListingsInput {
    pub market: Option<MarketSlug>,
    pub crop: Option<Crop>,
    pub status: ListingStatus,
    pub pagination: Pagination,
}

pub struct BrowseListingsUseCase {
    repository: Arc<dyn AlwaRepository>,
}

impl BrowseListingsUseCase {
    pub fn new(repository: Arc<dyn AlwaRepository>) -> Self {
        Self { repository }
    }

    /// Returns one page of the public board, newest first, and how many
    /// listings match in all.
    pub async fn execute(
        &self,
        input: BrowseListingsInput,
    ) -> Result<(Vec<ListingCard>, u64), AppError> {
        let market_id = match &input.market {
            Some(slug) => {
                let Some(market) = self.repository.find_market_by_slug(slug).await? else {
                    tracing::info!(market = slug.as_str(), "listings refused: no such alwa");

                    return Err(GlobalAppError::NotFound.into());
                };

                Some(*market.id())
            }
            None => None,
        };

        let now = Utc::now();
        let filter = ListingFilter {
            market_id,
            crop: input.crop,
            status: input.status,
        };

        let (listings, count) = self
            .repository
            .find_listings(&filter, now, &input.pagination)
            .await?;

        let cards = assemble_cards(self.repository.as_ref(), listings, now).await?;

        tracing::debug!(returned = cards.len(), count, "alwa listings browsed");

        Ok((cards, count))
    }
}

#[cfg(test)]
mod tests {
    use chrono::Duration;

    use super::*;
    use crate::features::alwa::{
        app::testing::{
            BUYER, FakeAlwaRepository, MARKET, MARKET_ID, RepositoryCall, a_listing, a_price,
            a_sold_listing, an_open_listing, an_open_offer, market_slug,
        },
        domain::FairPrice,
    };

    fn input(status: ListingStatus) -> BrowseListingsInput {
        BrowseListingsInput {
            market: None,
            crop: None,
            status,
            pagination: Pagination::new(1, 20),
        }
    }

    #[tokio::test]
    async fn shows_open_listings_with_their_offers_and_fair_price() {
        let listing = an_open_listing(7);
        let repository = FakeAlwaRepository::new()
            .with_offer(an_open_offer(1, &listing, BUYER, 950))
            .with_price(a_price(
                MARKET_ID,
                Crop::of("tomato"),
                Utc::now().date_naive(),
                1_000,
                false,
            ))
            .with_listing(listing);
        let use_case = BrowseListingsUseCase::new(Arc::new(repository.clone()));

        let (cards, count) = use_case
            .execute(input(ListingStatus::Open))
            .await
            .expect("board");

        assert_eq!(count, 1);
        assert_eq!(cards[0].open_offers(), 1);
        assert_eq!(cards[0].best_offer().map(|price| price.value()), Some(950));
        assert_eq!(*cards[0].fair_price(), FairPrice::Fair);
    }

    #[tokio::test]
    async fn a_listing_past_its_closing_time_is_on_the_closed_board_not_the_open_one() {
        let repository =
            FakeAlwaRepository::new().with_listing(a_listing(7, Utc::now() - Duration::days(5)));
        let use_case = BrowseListingsUseCase::new(Arc::new(repository.clone()));

        let (open, _) = use_case
            .execute(input(ListingStatus::Open))
            .await
            .expect("board");
        let (closed, _) = use_case
            .execute(input(ListingStatus::Closed))
            .await
            .expect("board");

        assert!(open.is_empty());
        assert_eq!(closed.len(), 1);
        assert_eq!(*closed[0].status(), ListingStatus::Closed);
    }

    #[tokio::test]
    async fn the_filters_and_the_page_reach_the_repository() {
        let (sold, _) = a_sold_listing(7);
        let repository = FakeAlwaRepository::new().with_listing(sold);
        let use_case = BrowseListingsUseCase::new(Arc::new(repository.clone()));

        use_case
            .execute(BrowseListingsInput {
                market: Some(market_slug(MARKET)),
                crop: Some(Crop::of("tomato")),
                status: ListingStatus::Sold,
                pagination: Pagination::new(2, 10),
            })
            .await
            .expect("board");

        assert!(repository.calls().contains(&RepositoryCall::FindListings {
            filter: ListingFilter {
                market_id: Some(MARKET_ID),
                crop: Some(Crop::of("tomato")),
                status: ListingStatus::Sold,
            },
            page: 2,
            rows_per_page: 10,
        }));
    }

    #[tokio::test]
    async fn an_empty_page_loads_no_offers_or_prices() {
        let repository = FakeAlwaRepository::new();
        let use_case = BrowseListingsUseCase::new(Arc::new(repository.clone()));

        let (cards, count) = use_case
            .execute(input(ListingStatus::Open))
            .await
            .expect("board");

        assert!(cards.is_empty());
        assert_eq!(count, 0);
        assert_eq!(repository.calls().len(), 1);
    }

    #[tokio::test]
    async fn an_unknown_alwa_is_not_found() {
        let use_case = BrowseListingsUseCase::new(Arc::new(FakeAlwaRepository::new()));

        let result = use_case
            .execute(BrowseListingsInput {
                market: Some(market_slug("baghdad")),
                ..input(ListingStatus::Open)
            })
            .await;

        assert!(matches!(
            result,
            Err(AppError::GlobalAppError(GlobalAppError::NotFound))
        ));
    }
}
