use std::sync::Arc;

use chrono::Utc;

use crate::{
    app::{AppError as GlobalAppError, Pagination},
    features::alwa::{
        app::{AlwaRepository, AppError, ListingFilter, listing_cards::assemble_cards},
        domain::{Crop, GeoPoint, ListingCard, ListingStatus, MarketSlug},
    },
};

pub struct BrowseListingsInput {
    pub market: Option<MarketSlug>,
    pub crop: Option<Crop>,
    pub status: ListingStatus,
    /// Where the reader stands. Given, the board is ordered nearest first.
    pub near: Option<GeoPoint>,
    pub pagination: Pagination,
}

pub struct BrowseListingsUseCase {
    repository: Arc<dyn AlwaRepository>,
}

impl BrowseListingsUseCase {
    pub fn new(repository: Arc<dyn AlwaRepository>) -> Self {
        Self { repository }
    }

    /// Returns one page of the public board and how many listings match in
    /// all: nearest first with its distance when the reader said where they
    /// stand, newest first otherwise.
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
            .find_listings(&filter, input.near.as_ref(), now, &input.pagination)
            .await?;

        let cards = assemble_cards(self.repository.as_ref(), listings, now).await?;
        let cards: Vec<ListingCard> = match &input.near {
            Some(from) => cards.into_iter().map(|card| card.seen_from(from)).collect(),
            None => cards,
        };

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
            BUYER, ERBIL, FakeAlwaRepository, MARKET, MARKET_ID, RepositoryCall, SULAYMANIYAH,
            a_listing, a_price, a_sold_listing, an_open_listing, an_open_offer, market_slug, point,
        },
        domain::FairPrice,
    };

    fn input(status: ListingStatus) -> BrowseListingsInput {
        BrowseListingsInput {
            market: None,
            crop: None,
            status,
            near: None,
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
                near: None,
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
            nearest_first: false,
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

    #[tokio::test]
    async fn from_a_point_the_nearest_listing_leads_and_each_tells_its_distance() {
        let in_sulaymaniyah =
            an_open_listing(1).placed_at(Some(point(SULAYMANIYAH.0, SULAYMANIYAH.1)));
        let unplaced = an_open_listing(2);
        let in_erbil = an_open_listing(3).placed_at(Some(point(ERBIL.0, ERBIL.1)));
        let repository = FakeAlwaRepository::new()
            .with_listing(in_sulaymaniyah)
            .with_listing(unplaced)
            .with_listing(in_erbil);
        let use_case = BrowseListingsUseCase::new(Arc::new(repository.clone()));

        let (cards, count) = use_case
            .execute(BrowseListingsInput {
                near: Some(point(ERBIL.0, ERBIL.1)),
                ..input(ListingStatus::Open)
            })
            .await
            .expect("board");

        assert_eq!(count, 3);
        assert_eq!(
            cards
                .iter()
                .map(|card| card.listing().id().unwrap_or_default())
                .collect::<Vec<_>>(),
            vec![3, 1, 2],
            "nearest first, the listing without a place last"
        );
        assert_eq!(*cards[0].distance_km(), Some(0.0));
        assert!(
            cards[1]
                .distance_km()
                .is_some_and(|km| (145.0..150.0).contains(&km))
        );
        assert_eq!(*cards[2].distance_km(), None);
        assert!(repository.calls().iter().any(|call| matches!(
            call,
            RepositoryCall::FindListings {
                nearest_first: true,
                ..
            }
        )));
    }

    #[tokio::test]
    async fn without_a_point_no_card_tells_a_distance() {
        let repository = FakeAlwaRepository::new()
            .with_listing(an_open_listing(1).placed_at(Some(point(ERBIL.0, ERBIL.1))));
        let use_case = BrowseListingsUseCase::new(Arc::new(repository.clone()));

        let (cards, _) = use_case
            .execute(input(ListingStatus::Open))
            .await
            .expect("board");

        assert_eq!(*cards[0].distance_km(), None);
    }
}
