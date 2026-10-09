use std::sync::Arc;

use chrono::{Duration, NaiveDate};

use crate::{
    app::AppError as GlobalAppError,
    features::alwa::{
        app::{AlwaRepository, AppError},
        domain::{Market, MarketSlug, PRICE_CHANGE_DAYS, Price},
    },
};

pub struct ViewMarketPricesInput {
    pub market: MarketSlug,
    /// `None` asks for the latest day the market has any price for.
    pub day: Option<NaiveDate>,
}

pub struct PriceOnBoard {
    pub price: Price,
    pub change_pct_7d: Option<i32>,
}

/// The price board of one alwa on one day.
pub struct MarketPrices {
    pub market: Market,
    /// `None` when no day was asked for and the market has no price at all.
    pub day: Option<NaiveDate>,
    pub prices: Vec<PriceOnBoard>,
}

pub struct ViewMarketPricesUseCase {
    repository: Arc<dyn AlwaRepository>,
}

impl ViewMarketPricesUseCase {
    pub fn new(repository: Arc<dyn AlwaRepository>) -> Self {
        Self { repository }
    }

    pub async fn execute(&self, input: ViewMarketPricesInput) -> Result<MarketPrices, AppError> {
        let Some(market) = self.repository.find_market_by_slug(&input.market).await? else {
            tracing::info!(
                market = input.market.as_str(),
                "prices refused: no such alwa"
            );

            return Err(GlobalAppError::NotFound.into());
        };

        let day = match input.day {
            Some(day) => Some(day),
            None => self.repository.find_latest_price_day(*market.id()).await?,
        };

        let Some(day) = day else {
            return Ok(MarketPrices {
                market,
                day: None,
                prices: Vec::new(),
            });
        };

        let prices = self.repository.find_prices_on(*market.id(), day).await?;
        let earlier = self
            .repository
            .find_prices_on(*market.id(), day - Duration::days(PRICE_CHANGE_DAYS))
            .await?;

        let prices: Vec<PriceOnBoard> = prices
            .into_iter()
            .map(|price| {
                let before = earlier.iter().find(|other| other.crop() == price.crop());

                PriceOnBoard {
                    change_pct_7d: price.change_pct_from(before),
                    price,
                }
            })
            .collect();

        tracing::debug!(
            market = market.slug().as_str(),
            %day,
            returned = prices.len(),
            "alwa prices viewed"
        );

        Ok(MarketPrices {
            market,
            day: Some(day),
            prices,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::features::alwa::{
        app::testing::{
            FakeAlwaRepository, MARKET, MARKET_ID, RepositoryCall, a_price, market_slug,
        },
        domain::Crop,
    };

    fn day(day_of_month: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(2026, 10, day_of_month).expect("day")
    }

    fn input(day: Option<NaiveDate>) -> ViewMarketPricesInput {
        ViewMarketPricesInput {
            market: market_slug(MARKET),
            day,
        }
    }

    fn a_board() -> FakeAlwaRepository {
        FakeAlwaRepository::new()
            .with_price(a_price(MARKET_ID, Crop::of("tomato"), day(1), 1_000, false))
            .with_price(a_price(MARKET_ID, Crop::of("tomato"), day(8), 1_250, false))
            .with_price(a_price(MARKET_ID, Crop::of("wheat"), day(1), 800, true))
            .with_price(a_price(MARKET_ID, Crop::of("wheat"), day(8), 850, true))
            .with_price(a_price(MARKET_ID, Crop::of("onion"), day(8), 600, false))
            .with_price(a_price(2, Crop::of("tomato"), day(9), 9_999, false))
    }

    #[tokio::test]
    async fn without_a_day_the_latest_day_with_a_price_at_that_market_is_shown() {
        let repository = a_board();
        let use_case = ViewMarketPricesUseCase::new(Arc::new(repository.clone()));

        let board = use_case.execute(input(None)).await.expect("board");

        assert_eq!(
            board.day,
            Some(day(8)),
            "a later price at another alwa does not count"
        );
        assert_eq!(board.prices.len(), 3);
        assert!(
            repository
                .calls()
                .contains(&RepositoryCall::FindLatestPriceDay {
                    market_id: MARKET_ID
                })
        );
    }

    #[tokio::test]
    async fn each_price_is_compared_with_the_one_seven_days_earlier() {
        let repository = a_board();
        let use_case = ViewMarketPricesUseCase::new(Arc::new(repository.clone()));

        let board = use_case.execute(input(Some(day(8)))).await.expect("board");

        let change_of = |crop: Crop| {
            board
                .prices
                .iter()
                .find(|row| *row.price.crop() == crop)
                .expect("row")
                .change_pct_7d
        };

        assert_eq!(change_of(Crop::of("tomato")), Some(25));
        assert_eq!(change_of(Crop::of("wheat")), None, "wheat is a fixed price");
        assert_eq!(
            change_of(Crop::of("onion")),
            None,
            "onion had no price a week ago"
        );
        assert_eq!(
            repository.calls(),
            vec![
                RepositoryCall::FindMarketBySlug {
                    slug: MARKET.to_string()
                },
                RepositoryCall::FindPricesOn {
                    market_id: MARKET_ID,
                    day: day(8)
                },
                RepositoryCall::FindPricesOn {
                    market_id: MARKET_ID,
                    day: day(1)
                },
            ]
        );
    }

    #[tokio::test]
    async fn a_market_without_any_price_shows_an_empty_board() {
        let use_case = ViewMarketPricesUseCase::new(Arc::new(FakeAlwaRepository::new()));

        let board = use_case.execute(input(None)).await.expect("board");

        assert_eq!(board.day, None);
        assert!(board.prices.is_empty());
    }

    #[tokio::test]
    async fn an_unknown_alwa_is_not_found() {
        let repository = a_board();
        let use_case = ViewMarketPricesUseCase::new(Arc::new(repository.clone()));

        let result = use_case
            .execute(ViewMarketPricesInput {
                market: market_slug("baghdad"),
                day: None,
            })
            .await;

        assert!(matches!(
            result,
            Err(AppError::GlobalAppError(GlobalAppError::NotFound))
        ));
        assert_eq!(repository.calls().len(), 1, "nothing is read after that");
    }
}
