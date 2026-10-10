use std::sync::Arc;

use chrono::NaiveDate;

use crate::{
    app::AppError as GlobalAppError,
    features::alwa::{
        app::{AlwaRepository, AppError, CropDirectory},
        domain::{Crop, Market, MarketSlug, Price, PricePerKg, PriceSource},
    },
};

pub struct RecordPriceInput {
    pub market: MarketSlug,
    pub crop: Crop,
    pub day: NaiveDate,
    pub price: PricePerKg,
    pub fixed: bool,
    pub source: PriceSource,
}

pub struct RecordPriceUseCase {
    repository: Arc<dyn AlwaRepository>,
    crops: Arc<dyn CropDirectory>,
}

impl RecordPriceUseCase {
    pub fn new(repository: Arc<dyn AlwaRepository>, crops: Arc<dyn CropDirectory>) -> Self {
        Self { repository, crops }
    }

    /// Stores the price a data job reports. Sending the same market, crop
    /// and day again replaces the earlier price.
    pub async fn execute(&self, input: RecordPriceInput) -> Result<(Market, Price), AppError> {
        // A price a job sends is new data, also when it replaces an earlier
        // one, so its crop must be one staff have switched on.
        let product =
            self.crops.active().await?.allow(input.crop).inspect_err(
                |error| tracing::info!(%error, "price refused: the crop is not in use"),
            )?;

        let Some(market) = self.repository.find_market_by_slug(&input.market).await? else {
            tracing::info!(
                market = input.market.as_str(),
                "price refused: no such alwa"
            );

            return Err(GlobalAppError::NotFound.into());
        };

        // The price is for one of the product's unit as it is today.
        let price = Price::new(
            &market,
            product,
            input.day,
            input.price,
            input.fixed,
            input.source,
        );

        let recorded = self.repository.upsert_price(&price).await?;

        tracing::info!(
            market = market.slug().as_str(),
            crop = String::from(*recorded.crop()),
            day = %recorded.day(),
            price_iqd_per_kg = recorded.price().value(),
            fixed = *recorded.fixed(),
            "alwa price recorded"
        );

        Ok((market, recorded))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::features::alwa::app::testing::{
        FakeAlwaRepository, FakeCropDirectory, MARKET, MARKET_ID, RepositoryCall, market_slug,
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
            fixed: true,
            source: PriceSource::new("ministry-of-trade".to_string()).expect("source"),
        }
    }

    #[tokio::test]
    async fn records_the_price_at_the_named_alwa() {
        let repository = FakeAlwaRepository::new();
        let use_case = RecordPriceUseCase::new(
            Arc::new(repository.clone()),
            Arc::new(FakeCropDirectory::seeded()),
        );

        let (market, price) = use_case.execute(input(850)).await.expect("price");

        assert_eq!(market.slug().as_str(), MARKET);
        assert_eq!(price.price().value(), 850);
        assert!(*price.fixed());
        assert_eq!(
            repository.calls(),
            vec![
                RepositoryCall::FindMarketBySlug {
                    slug: MARKET.to_string()
                },
                RepositoryCall::UpsertPrice {
                    market_id: MARKET_ID,
                    crop: Crop::of("wheat"),
                    day: day(),
                },
            ]
        );
    }

    #[tokio::test]
    async fn sending_the_same_day_again_replaces_the_price() {
        let repository = FakeAlwaRepository::new();
        let use_case = RecordPriceUseCase::new(
            Arc::new(repository.clone()),
            Arc::new(FakeCropDirectory::seeded()),
        );

        use_case.execute(input(850)).await.expect("first");
        use_case.execute(input(870)).await.expect("second");

        let stored = repository.stored_prices();

        assert_eq!(stored.len(), 1, "one price per market, crop and day");
        assert_eq!(stored[0].price().value(), 870);
    }

    #[tokio::test]
    async fn a_price_for_an_unknown_alwa_is_not_written() {
        let repository = FakeAlwaRepository::new();
        let use_case = RecordPriceUseCase::new(
            Arc::new(repository.clone()),
            Arc::new(FakeCropDirectory::seeded()),
        );

        let result = use_case
            .execute(RecordPriceInput {
                market: market_slug("baghdad"),
                ..input(850)
            })
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
            RecordPriceUseCase::new(Arc::new(repository.clone()), Arc::new(crops.clone()));

        let result = use_case.execute(input(850)).await;

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
        let use_case = RecordPriceUseCase::new(
            Arc::new(repository.clone()),
            Arc::new(FakeCropDirectory::failing()),
        );

        assert!(use_case.execute(input(850)).await.is_err());
        assert!(repository.calls().is_empty());
    }
}
