use std::sync::Arc;

use chrono::NaiveDate;

use crate::{
    app::{AppError as GlobalAppError, Pagination},
    features::alwa::{
        app::{AlwaRepository, AppError, StoredPriceFilter},
        domain::{Crop, Market, MarketSlug, Price},
    },
    shared::DomainError,
};

pub struct ListStoredPricesInput {
    pub market: MarketSlug,
    pub crop: Option<Crop>,
    pub from: Option<NaiveDate>,
    pub to: Option<NaiveDate>,
    pub pagination: Pagination,
}

pub struct ListStoredPricesUseCase {
    repository: Arc<dyn AlwaRepository>,
}

impl ListStoredPricesUseCase {
    pub fn new(repository: Arc<dyn AlwaRepository>) -> Self {
        Self { repository }
    }

    /// Returns one page of the prices stored for a market exactly as they
    /// are, newest day first, and how many match in all.
    pub async fn execute(
        &self,
        input: ListStoredPricesInput,
    ) -> Result<(Market, Vec<Price>, u64), AppError> {
        if let (Some(from), Some(to)) = (input.from, input.to)
            && from > to
        {
            return Err(
                DomainError::InvalidValue("`from` must not be after `to`".to_string()).into(),
            );
        }

        let Some(market) = self.repository.find_market_by_slug(&input.market).await? else {
            tracing::info!(
                market = input.market.as_str(),
                "stored prices refused: no such alwa"
            );

            return Err(GlobalAppError::NotFound.into());
        };

        let filter = StoredPriceFilter {
            market_id: *market.id(),
            crop: input.crop,
            from: input.from,
            to: input.to,
        };

        let (prices, count) = self
            .repository
            .find_stored_prices(&filter, &input.pagination)
            .await?;

        tracing::debug!(returned = prices.len(), count, "stored alwa prices listed");

        Ok((market, prices, count))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::features::alwa::app::testing::{
        FakeAlwaRepository, MARKET, MARKET_ID, RepositoryCall, a_price, market_slug,
    };

    fn day(day: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(2026, 10, day).expect("day")
    }

    fn input() -> ListStoredPricesInput {
        ListStoredPricesInput {
            market: market_slug(MARKET),
            crop: None,
            from: None,
            to: None,
            pagination: Pagination::new(1, 20),
        }
    }

    fn repository() -> FakeAlwaRepository {
        FakeAlwaRepository::new()
            .with_price(a_price(MARKET_ID, Crop::Tomato, day(1), 900, false))
            .with_price(a_price(MARKET_ID, Crop::Tomato, day(3), 950, false))
            .with_price(a_price(MARKET_ID, Crop::Wheat, day(2), 850, true))
            .with_price(a_price(2, Crop::Tomato, day(3), 700, false))
    }

    #[tokio::test]
    async fn lists_only_the_markets_prices_newest_day_first() {
        let repository = repository();
        let use_case = ListStoredPricesUseCase::new(Arc::new(repository.clone()));

        let (market, prices, count) = use_case.execute(input()).await.expect("prices");

        assert_eq!(market.slug().as_str(), MARKET);
        assert_eq!(count, 3);
        assert_eq!(
            prices.iter().map(|price| *price.day()).collect::<Vec<_>>(),
            vec![day(3), day(2), day(1)]
        );
        assert_eq!(
            repository.calls()[1],
            RepositoryCall::FindStoredPrices {
                filter: StoredPriceFilter {
                    market_id: MARKET_ID,
                    crop: None,
                    from: None,
                    to: None,
                },
                page: 1,
                rows_per_page: 20,
            }
        );
    }

    #[tokio::test]
    async fn narrows_by_crop_and_by_days_both_ends_included() {
        let use_case = ListStoredPricesUseCase::new(Arc::new(repository()));

        let (_, prices, count) = use_case
            .execute(ListStoredPricesInput {
                crop: Some(Crop::Tomato),
                from: Some(day(1)),
                to: Some(day(3)),
                ..input()
            })
            .await
            .expect("prices");

        assert_eq!(count, 2);
        assert!(prices.iter().all(|price| *price.crop() == Crop::Tomato));
    }

    #[tokio::test]
    async fn the_count_is_of_every_page() {
        let use_case = ListStoredPricesUseCase::new(Arc::new(repository()));

        let (_, prices, count) = use_case
            .execute(ListStoredPricesInput {
                pagination: Pagination::new(2, 2),
                ..input()
            })
            .await
            .expect("prices");

        assert_eq!(count, 3);
        assert_eq!(prices.len(), 1);
    }

    #[tokio::test]
    async fn days_the_wrong_way_round_are_refused_before_any_query() {
        let repository = repository();
        let use_case = ListStoredPricesUseCase::new(Arc::new(repository.clone()));

        let result = use_case
            .execute(ListStoredPricesInput {
                from: Some(day(3)),
                to: Some(day(1)),
                ..input()
            })
            .await;

        assert!(matches!(result, Err(AppError::Domain(_))));
        assert!(repository.calls().is_empty());
    }

    #[tokio::test]
    async fn an_unknown_market_is_not_found() {
        let use_case = ListStoredPricesUseCase::new(Arc::new(repository()));

        let result = use_case
            .execute(ListStoredPricesInput {
                market: market_slug("baghdad"),
                ..input()
            })
            .await;

        assert!(matches!(
            result,
            Err(AppError::GlobalAppError(GlobalAppError::NotFound))
        ));
    }
}
