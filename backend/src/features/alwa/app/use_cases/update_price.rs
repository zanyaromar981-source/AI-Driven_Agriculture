use std::sync::Arc;

use crate::{
    app::AppError as GlobalAppError,
    features::alwa::{
        app::{AlwaRepository, AppError, CropDirectory, use_cases::RecordPriceInput},
        domain::{Market, Price},
    },
};

pub struct UpdatePriceUseCase {
    repository: Arc<dyn AlwaRepository>,
    crops: Arc<dyn CropDirectory>,
}

impl UpdatePriceUseCase {
    pub fn new(repository: Arc<dyn AlwaRepository>, crops: Arc<dyn CropDirectory>) -> Self {
        Self { repository, crops }
    }

    /// A staff member corrects a stored price. Unlike the data job's upsert
    /// it never creates: a day with no price is not found.
    pub async fn execute(
        &self,
        staff_id: i32,
        input: RecordPriceInput,
    ) -> Result<(Market, Price), AppError> {
        // A changed price is new data, so its crop must be one staff have
        // switched on. A price of a crop switched off since stays as it is.
        self.crops
            .active()
            .await?
            .allow(input.crop)
            .inspect_err(|error| tracing::info!(%error, "price refused: the crop is not in use"))?;

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
        app::testing::{
            FakeAlwaRepository, FakeCropDirectory, MARKET, MARKET_ID, a_price, market_slug,
        },
        domain::{Crop, PricePerKg, PriceSource},
    };

    fn day() -> NaiveDate {
        NaiveDate::from_ymd_opt(2026, 10, 8).expect("day")
    }

    fn input(price: i64) -> RecordPriceInput {
        RecordPriceInput {
            market: market_slug(MARKET),
            crop: Crop::of("wheat"),
            day: day(),
            price: PricePerKg::new(price).expect("price"),
            fixed: false,
            source: PriceSource::new("ministry desk".to_string()).expect("source"),
        }
    }

    #[tokio::test]
    async fn replaces_every_field_of_the_stored_price_but_its_key() {
        let repository = FakeAlwaRepository::new().with_price(a_price(
            MARKET_ID,
            Crop::of("wheat"),
            day(),
            850,
            true,
        ));
        let use_case = UpdatePriceUseCase::new(
            Arc::new(repository.clone()),
            Arc::new(FakeCropDirectory::seeded()),
        );

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
        let use_case = UpdatePriceUseCase::new(
            Arc::new(repository.clone()),
            Arc::new(FakeCropDirectory::seeded()),
        );

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
        let use_case = UpdatePriceUseCase::new(
            Arc::new(repository.clone()),
            Arc::new(FakeCropDirectory::seeded()),
        );

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

    #[tokio::test]
    async fn a_crop_that_is_unknown_or_switched_off_is_refused_by_name_and_nothing_is_written() {
        let repository = FakeAlwaRepository::new();
        // The price is for wheat, and wheat is not in the list.
        let crops = FakeCropDirectory::with(&["tomato"]);
        let use_case =
            UpdatePriceUseCase::new(Arc::new(repository.clone()), Arc::new(crops.clone()));

        let result = use_case.execute(9, input(900)).await;

        assert!(matches!(
            result,
            Err(AppError::Alwa(crate::features::alwa::domain::AlwaError::UnknownCrop(code)))
                if code == "wheat"
        ));
        assert_eq!(crops.asked(), 1);
        assert!(
            repository.calls().is_empty(),
            "the crop is checked before anything is read or written"
        );
    }

    #[tokio::test]
    async fn when_the_crop_list_cannot_be_read_nothing_is_written() {
        let repository = FakeAlwaRepository::new();
        let use_case = UpdatePriceUseCase::new(
            Arc::new(repository.clone()),
            Arc::new(FakeCropDirectory::failing()),
        );

        assert!(use_case.execute(9, input(900)).await.is_err());
        assert!(repository.calls().is_empty());
    }
}
