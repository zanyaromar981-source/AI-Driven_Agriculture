use std::sync::Arc;

use chrono::{Duration, Utc};

use crate::{
    app::AppError as GlobalAppError,
    features::alwa::{
        app::{AlwaRepository, AppError},
        domain::{Crop, HistoryDays, Market, MarketSlug, Price},
    },
};

pub struct ViewPriceHistoryInput {
    pub market: MarketSlug,
    pub crop: Crop,
    pub days: HistoryDays,
}

pub struct ViewPriceHistoryUseCase {
    repository: Arc<dyn AlwaRepository>,
}

impl ViewPriceHistoryUseCase {
    pub fn new(repository: Arc<dyn AlwaRepository>) -> Self {
        Self { repository }
    }

    /// Returns the prices of the crop over the last `days` days, today (UTC)
    /// included, oldest first. Days without a price are left out.
    pub async fn execute(
        &self,
        input: ViewPriceHistoryInput,
    ) -> Result<(Market, Vec<Price>), AppError> {
        let Some(market) = self.repository.find_market_by_slug(&input.market).await? else {
            tracing::info!(
                market = input.market.as_str(),
                "history refused: no such alwa"
            );

            return Err(GlobalAppError::NotFound.into());
        };

        let today = Utc::now().date_naive();
        let from = today - Duration::days(i64::from(input.days.value()) - 1);

        let history = self
            .repository
            .find_prices_between(&[*market.id()], &[input.crop], from, today)
            .await?;

        tracing::debug!(
            market = market.slug().as_str(),
            days = input.days.value(),
            returned = history.len(),
            "alwa price history viewed"
        );

        Ok((market, history))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::features::alwa::app::testing::{
        FakeAlwaRepository, MARKET, MARKET_ID, RepositoryCall, a_price, market_slug,
    };

    fn input(days: u32) -> ViewPriceHistoryInput {
        ViewPriceHistoryInput {
            market: market_slug(MARKET),
            crop: Crop::Tomato,
            days: HistoryDays::new(days).expect("days"),
        }
    }

    #[tokio::test]
    async fn returns_the_window_ending_today_oldest_first() {
        let today = Utc::now().date_naive();
        let repository = FakeAlwaRepository::new()
            .with_price(a_price(MARKET_ID, Crop::Tomato, today, 1_200, false))
            .with_price(a_price(
                MARKET_ID,
                Crop::Tomato,
                today - Duration::days(6),
                1_000,
                false,
            ))
            .with_price(a_price(
                MARKET_ID,
                Crop::Tomato,
                today - Duration::days(7),
                900,
                false,
            ))
            .with_price(a_price(MARKET_ID, Crop::Onion, today, 600, false));
        let use_case = ViewPriceHistoryUseCase::new(Arc::new(repository.clone()));

        let (market, history) = use_case.execute(input(7)).await.expect("history");

        assert_eq!(market.slug().as_str(), MARKET);
        assert_eq!(
            history
                .iter()
                .map(|price| price.price().value())
                .collect::<Vec<_>>(),
            vec![1_000, 1_200],
            "seven days are today and the six before it"
        );
        assert!(
            repository
                .calls()
                .contains(&RepositoryCall::FindPricesBetween {
                    market_ids: vec![MARKET_ID],
                    crops: vec![Crop::Tomato],
                    from: today - Duration::days(6),
                    to: today,
                })
        );
    }

    #[tokio::test]
    async fn one_day_of_history_is_today_alone() {
        let today = Utc::now().date_naive();
        let repository = FakeAlwaRepository::new();
        let use_case = ViewPriceHistoryUseCase::new(Arc::new(repository.clone()));

        use_case.execute(input(1)).await.expect("history");

        assert!(
            repository
                .calls()
                .contains(&RepositoryCall::FindPricesBetween {
                    market_ids: vec![MARKET_ID],
                    crops: vec![Crop::Tomato],
                    from: today,
                    to: today,
                })
        );
    }

    #[tokio::test]
    async fn an_unknown_alwa_is_not_found() {
        let use_case = ViewPriceHistoryUseCase::new(Arc::new(FakeAlwaRepository::new()));

        let result = use_case
            .execute(ViewPriceHistoryInput {
                market: market_slug("baghdad"),
                ..input(7)
            })
            .await;

        assert!(matches!(
            result,
            Err(AppError::GlobalAppError(GlobalAppError::NotFound))
        ));
    }
}
