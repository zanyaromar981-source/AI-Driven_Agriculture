use std::sync::Arc;

use chrono::NaiveDate;

use crate::features::alwa::{
    app::{AlwaRepository, AppError},
    domain::{Crop, MarketSlug},
};

pub struct DeletePriceUseCase {
    repository: Arc<dyn AlwaRepository>,
}

impl DeletePriceUseCase {
    pub fn new(repository: Arc<dyn AlwaRepository>) -> Self {
        Self { repository }
    }

    /// A staff member removes one stored price. Removing one that is already
    /// gone, or of a market that is gone, succeeds: gone is what was asked.
    pub async fn execute(
        &self,
        staff_id: i32,
        market: MarketSlug,
        crop: Crop,
        day: NaiveDate,
    ) -> Result<(), AppError> {
        let removed = self.repository.delete_price(&market, crop, day).await?;

        tracing::info!(
            staff_id,
            market = market.as_str(),
            crop = String::from(crop),
            %day,
            removed,
            "alwa price deleted"
        );

        Ok(())
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

    #[tokio::test]
    async fn removes_exactly_the_named_price() {
        let repository = FakeAlwaRepository::new()
            .with_price(a_price(MARKET_ID, Crop::of("tomato"), day(1), 900, false))
            .with_price(a_price(MARKET_ID, Crop::of("tomato"), day(2), 950, false))
            .with_price(a_price(2, Crop::of("tomato"), day(1), 700, false));
        let use_case = DeletePriceUseCase::new(Arc::new(repository.clone()));

        use_case
            .execute(9, market_slug(MARKET), Crop::of("tomato"), day(1))
            .await
            .expect("delete");

        let left = repository.stored_prices();

        assert_eq!(left.len(), 2);
        assert!(
            left.iter()
                .all(|price| (*price.market_id(), *price.day()) != (MARKET_ID, day(1)))
        );
        assert_eq!(
            repository.calls(),
            vec![RepositoryCall::DeletePrice {
                market: MARKET.to_string(),
                crop: Crop::of("tomato"),
                day: day(1),
            }],
            "one statement, with no lookup before it"
        );
    }

    #[tokio::test]
    async fn deleting_what_is_gone_succeeds() {
        let use_case = DeletePriceUseCase::new(Arc::new(FakeAlwaRepository::new()));

        for market in [MARKET, "baghdad"] {
            assert!(
                use_case
                    .execute(9, market_slug(market), Crop::of("tomato"), day(1))
                    .await
                    .is_ok()
            );
        }
    }

    #[tokio::test]
    async fn a_repository_failure_surfaces() {
        let use_case = DeletePriceUseCase::new(Arc::new(FakeAlwaRepository::failing()));

        assert!(
            use_case
                .execute(9, market_slug(MARKET), Crop::of("tomato"), day(1))
                .await
                .is_err()
        );
    }
}
