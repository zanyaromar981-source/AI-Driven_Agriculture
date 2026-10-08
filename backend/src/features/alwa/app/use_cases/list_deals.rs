use std::sync::Arc;

use chrono::{NaiveDate, Utc};

use crate::{
    app::AppError as GlobalAppError,
    features::alwa::{
        app::{AlwaRepository, AppError},
        domain::{Deal, MarketSlug},
    },
};

pub struct ListDealsInput {
    pub market: Option<MarketSlug>,
    /// `None` asks for today (UTC).
    pub day: Option<NaiveDate>,
}

pub struct ListDealsUseCase {
    repository: Arc<dyn AlwaRepository>,
}

impl ListDealsUseCase {
    pub fn new(repository: Arc<dyn AlwaRepository>) -> Self {
        Self { repository }
    }

    /// Returns the day that was read and the deals made on it.
    pub async fn execute(&self, input: ListDealsInput) -> Result<(NaiveDate, Vec<Deal>), AppError> {
        let market_id = match &input.market {
            Some(slug) => {
                let Some(market) = self.repository.find_market_by_slug(slug).await? else {
                    tracing::info!(market = slug.as_str(), "deals refused: no such alwa");

                    return Err(GlobalAppError::NotFound.into());
                };

                Some(*market.id())
            }
            None => None,
        };

        let day = input.day.unwrap_or_else(|| Utc::now().date_naive());
        let deals = self.repository.find_deals(market_id, day).await?;

        tracing::debug!(%day, returned = deals.len(), "alwa deals listed");

        Ok((day, deals))
    }
}

#[cfg(test)]
mod tests {
    use chrono::Duration;

    use super::*;
    use crate::features::alwa::app::testing::{
        FakeAlwaRepository, MARKET, MARKET_ID, RepositoryCall, a_sold_listing, an_open_listing,
        an_open_offer, market_slug,
    };

    fn a_day_of_trade() -> FakeAlwaRepository {
        let (sold, offers) = a_sold_listing(7);
        let open = an_open_listing(8);

        let mut repository = FakeAlwaRepository::new()
            .with_offer(an_open_offer(3, &open, "+9647701112233", 990))
            .with_listing(sold)
            .with_listing(open);

        for offer in offers {
            repository = repository.with_offer(offer);
        }

        repository
    }

    #[tokio::test]
    async fn without_a_day_todays_deals_are_listed() {
        let repository = a_day_of_trade();
        let use_case = ListDealsUseCase::new(Arc::new(repository.clone()));

        let (day, deals) = use_case
            .execute(ListDealsInput {
                market: None,
                day: None,
            })
            .await
            .expect("deals");

        assert_eq!(day, Utc::now().date_naive());
        assert_eq!(deals.len(), 1, "only the accepted offer is a deal");
        assert_eq!(deals[0].offer().price().value(), 950);
        assert_eq!(
            repository.calls(),
            vec![RepositoryCall::FindDeals {
                market_id: None,
                day
            }]
        );
    }

    #[tokio::test]
    async fn the_market_and_the_day_narrow_the_list() {
        let repository = a_day_of_trade();
        let use_case = ListDealsUseCase::new(Arc::new(repository.clone()));
        let yesterday = Utc::now().date_naive() - Duration::days(1);

        let (day, deals) = use_case
            .execute(ListDealsInput {
                market: Some(market_slug(MARKET)),
                day: Some(yesterday),
            })
            .await
            .expect("deals");

        assert_eq!(day, yesterday);
        assert!(deals.is_empty());
        assert!(repository.calls().contains(&RepositoryCall::FindDeals {
            market_id: Some(MARKET_ID),
            day: yesterday
        }));
    }

    #[tokio::test]
    async fn an_unknown_alwa_is_not_found() {
        let use_case = ListDealsUseCase::new(Arc::new(FakeAlwaRepository::new()));

        let result = use_case
            .execute(ListDealsInput {
                market: Some(market_slug("baghdad")),
                day: None,
            })
            .await;

        assert!(matches!(
            result,
            Err(AppError::GlobalAppError(GlobalAppError::NotFound))
        ));
    }
}
