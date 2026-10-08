use chrono::{DateTime, Duration, Utc};

use crate::features::alwa::{
    app::{AlwaRepository, AppError},
    domain::{Crop, Listing, ListingCard, REFERENCE_PRICE_LOOKBACK_DAYS},
};

/// Turns listings into the cards the board shows. The offers and the
/// reference prices of all of them are loaded in two queries, not two per
/// listing.
pub(super) async fn assemble_cards(
    repository: &dyn AlwaRepository,
    listings: Vec<Listing>,
    now: DateTime<Utc>,
) -> Result<Vec<ListingCard>, AppError> {
    let days = listings
        .iter()
        .map(|listing| listing.created_at().date_naive());

    let (Some(first_day), Some(last_day)) = (days.clone().min(), days.max()) else {
        return Ok(Vec::new());
    };

    let mut listing_ids = Vec::new();
    let mut market_ids: Vec<i32> = Vec::new();
    let mut crops: Vec<Crop> = Vec::new();

    for listing in &listings {
        listing_ids.push(listing.id().unwrap_or_default());

        if !market_ids.contains(listing.market_id()) {
            market_ids.push(*listing.market_id());
        }

        if !crops.contains(listing.crop()) {
            crops.push(*listing.crop());
        }
    }

    let offers = repository.find_offers_by_listings(&listing_ids).await?;
    let prices = repository
        .find_prices_between(
            &market_ids,
            &crops,
            first_day - Duration::days(REFERENCE_PRICE_LOOKBACK_DAYS),
            last_day,
        )
        .await?;

    Ok(listings
        .into_iter()
        .map(|listing| ListingCard::assemble(listing, &offers, &prices, now))
        .collect())
}
