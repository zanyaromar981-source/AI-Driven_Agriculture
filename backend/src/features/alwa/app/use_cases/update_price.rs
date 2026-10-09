use std::sync::Arc;

use crate::{
    app::AppError as GlobalAppError,
    features::alwa::{
        app::{AlwaRepository, AppError, use_cases::RecordPriceInput},
        domain::{Market, Price},
    },
};

pub struct UpdatePriceUseCase {
    repository: Arc<dyn AlwaRepository>,
}

impl UpdatePriceUseCase {
    pub fn new(repository: Arc<dyn AlwaRepository>) -> Self {
        Self { repository }
    }

    /// A staff member corrects a stored price. Unlike the data job's upsert
    /// it never creates: a day with no price is not found.
    pub async fn execute(
        &self,
        staff_id: i32,
        input: RecordPriceInput,
    ) -> Result<(Market, Price), AppError> {
        let Some(market) = self.repository.find_market_by_slug(&input.market).await? else {
            tracing::info!(
                staff_id,
                market = input.market.as_str(),
                "price update refused: no such alwa"
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

        let Some(updated) = self.repository.update_price(&price).await? else {
            tracing::info!(
                staff_id,
                market = market.slug().as_str(),
                crop = String::from(input.crop),
                day = %input.day,
                "price update refused: that day has none"
            );

            return Err(GlobalAppError::NotFound.into());
        };

        tracing::info!(
            staff_id,
            market = market.slug().as_str(),
            crop = String::from(*updated.crop()),
            day = %updated.day(),
            price_iqd_per_kg = updated.price().value(),
            fixed = *updated.fixed(),
            "alwa price updated"
        );

        Ok((market, updated))
    }
}

#[cfg(test)]
mod tests {
    use chrono::NaiveDate;

    use super::*;
    use crate::features::alwa::{
        app::testing::{FakeAlwaRepository, MARKET, MARKET_ID, a_price, market_slug},
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
            fixed: false,
            source: PriceSource::new("ministry desk".to_string()).expect("source"),
        }
    }

    #[tokio::test]
    async fn replaces_every_field_of_the_stored_price_but_its_key() {
        let repository =
            FakeAlwaRepository::new().with_price(a_price(MARKET_ID, Crop::Wheat, day(), 850, true));
        let use_case = UpdatePriceUseCase::new(Arc::new(repository.clone()));

        let (_, price) = use_case.execute(9, input(870)).await.expect("price");

        assert_eq!(price.price().value(), 870);
        assert!(!*price.fixed());
        assert_eq!(price.source().as_str(), "ministry desk");

        let stored = repository.stored_prices();

        assert_eq!(stored.len(), 1);
        assert_eq!(stored[0].price().value(), 870);
        assert_eq!(*stored[0].day(), day());
    }

    #[tokio::test]
    async fn a_day_with_no_price_is_not_found_and_gets_none() {
        let repository = FakeAlwaRepository::new();
        let use_case = UpdatePriceUseCase::new(Arc::new(repository.clone()));

        let result = use_case.execute(9, input(870)).await;

        assert!(matches!(
            result,
            Err(AppError::GlobalAppError(GlobalAppError::NotFound))
        ));
        assert!(
            repository.stored_prices().is_empty(),
            "update must never create"
        );
    }

    #[tokio::test]
    async fn an_unknown_alwa_is_not_found() {
        let repository = FakeAlwaRepository::new();
        let use_case = UpdatePriceUseCase::new(Arc::new(repository.clone()));

        let result = use_case
            .execute(
                9,
                RecordPriceInput {
                    market: market_slug("baghdad"),
                    ..input(870)
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
