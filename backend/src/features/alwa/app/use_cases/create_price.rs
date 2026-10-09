use std::sync::Arc;

use crate::{
    app::AppError as GlobalAppError,
    features::alwa::{
        app::{AlwaRepository, AppError, use_cases::RecordPriceInput},
        domain::{AlwaError, Market, Price},
    },
};

pub struct CreatePriceUseCase {
    repository: Arc<dyn AlwaRepository>,
}

impl CreatePriceUseCase {
    pub fn new(repository: Arc<dyn AlwaRepository>) -> Self {
        Self { repository }
    }

    /// A staff member enters a price by hand. Unlike the data job's upsert
    /// it never replaces: a price already stored for that market, crop and
    /// day is refused and left as it was.
    pub async fn execute(
        &self,
        staff_id: i32,
        input: RecordPriceInput,
    ) -> Result<(Market, Price), AppError> {
        let Some(market) = self.repository.find_market_by_slug(&input.market).await? else {
            tracing::info!(
                staff_id,
                market = input.market.as_str(),
                "price refused: no such alwa"
            );

            return Err(GlobalAppError::NotFound.into());
        };

        let price = Price::new(
            &market,
            input.crop,
            input.day,
            input.price,
            input.fixed,
            input.source,
        );

        let Some(created) = self.repository.create_price(&price).await? else {
            tracing::info!(
                staff_id,
                market = market.slug().as_str(),
                crop = String::from(input.crop),
                day = %input.day,
                "price refused: that day already has one"
            );

            return Err(AlwaError::AlreadyExists("price").into());
        };

        tracing::info!(
            staff_id,
            market = market.slug().as_str(),
            crop = String::from(*created.crop()),
            day = %created.day(),
            price_iqd_per_kg = created.price().value(),
            fixed = *created.fixed(),
            "alwa price created"
        );

        Ok((market, created))
    }
}

#[cfg(test)]
mod tests {
    use chrono::NaiveDate;

    use super::*;
    use crate::features::alwa::{
        app::testing::{FakeAlwaRepository, MARKET, MARKET_ID, RepositoryCall, market_slug},
        domain::{Crop, PricePerKg, PriceSource},
    };

    fn day() -> NaiveDate {
        NaiveDate::from_ymd_opt(2026, 10, 8).expect("day")
    }

    fn input(price: i64) -> RecordPriceInput {
        RecordPriceInput {
            market: market_slug(MARKET),
            crop: Crop::Wheat,
            day: day(),
            price: PricePerKg::new(price).expect("price"),
            fixed: true,
            source: PriceSource::new("ministry desk".to_string()).expect("source"),
        }
    }

    #[tokio::test]
    async fn stores_a_price_for_a_day_that_has_none() {
        let repository = FakeAlwaRepository::new();
        let use_case = CreatePriceUseCase::new(Arc::new(repository.clone()));

        let (market, price) = use_case.execute(9, input(850)).await.expect("price");

        assert_eq!(market.slug().as_str(), MARKET);
        assert_eq!(price.price().value(), 850);
        assert_eq!(price.source().as_str(), "ministry desk");
        assert_eq!(
            repository.calls(),
            vec![
                RepositoryCall::FindMarketBySlug {
                    slug: MARKET.to_string()
                },
                RepositoryCall::CreatePrice {
                    market_id: MARKET_ID,
                    crop: Crop::Wheat,
                    day: day(),
                },
            ]
        );
    }

    #[tokio::test]
    async fn a_second_price_for_the_same_day_is_refused_and_replaces_nothing() {
        let repository = FakeAlwaRepository::new();
        let use_case = CreatePriceUseCase::new(Arc::new(repository.clone()));

        use_case.execute(9, input(850)).await.expect("first");
        let second = use_case.execute(9, input(870)).await;

        assert!(matches!(
            second,
            Err(AppError::Alwa(AlwaError::AlreadyExists("price")))
        ));

        let stored = repository.stored_prices();

        assert_eq!(stored.len(), 1);
        assert_eq!(stored[0].price().value(), 850, "create must never replace");
    }

    #[tokio::test]
    async fn a_price_for_an_unknown_alwa_is_not_written() {
        let repository = FakeAlwaRepository::new();
        let use_case = CreatePriceUseCase::new(Arc::new(repository.clone()));

        let result = use_case
            .execute(
                9,
                RecordPriceInput {
                    market: market_slug("baghdad"),
                    ..input(850)
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
