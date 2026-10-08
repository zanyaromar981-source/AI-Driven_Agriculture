use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};
use validator::Validate;

use crate::{
    app::AppError as GlobalAppError,
    features::alwa::{
        app::{
            AppError,
            use_cases::{
                BrowseListingsInput, ListDealsInput, MarketPrices, PostListingInput, PriceOnBoard,
                RecordPriceInput, ViewMarketPricesInput, ViewPriceHistoryInput,
            },
        },
        domain::{
            self, Deal, DealsSummary, DisplayName, HistoryDays, IdempotencyKey, Listing,
            ListingCard, ListingDraft, Market, MarketSlug, Note, Offer, OfferDraft, PlacedOffer,
            Price, PricePerKg, PriceSource, QuantityKg, ZoneSlug,
        },
    },
    shared::{DomainError, Phone},
};

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum AlwaCrop {
    Wheat,
    Barley,
    Tomato,
    Cucumber,
    Potato,
    Onion,
    Watermelon,
    Grape,
    Olive,
    Sunflower,
    Chickpea,
    Pomegranate,
    Okra,
    Eggplant,
    Pepper,
    Apple,
}

impl From<AlwaCrop> for domain::Crop {
    fn from(value: AlwaCrop) -> Self {
        match value {
            AlwaCrop::Wheat => domain::Crop::Wheat,
            AlwaCrop::Barley => domain::Crop::Barley,
            AlwaCrop::Tomato => domain::Crop::Tomato,
            AlwaCrop::Cucumber => domain::Crop::Cucumber,
            AlwaCrop::Potato => domain::Crop::Potato,
            AlwaCrop::Onion => domain::Crop::Onion,
            AlwaCrop::Watermelon => domain::Crop::Watermelon,
            AlwaCrop::Grape => domain::Crop::Grape,
            AlwaCrop::Olive => domain::Crop::Olive,
            AlwaCrop::Sunflower => domain::Crop::Sunflower,
            AlwaCrop::Chickpea => domain::Crop::Chickpea,
            AlwaCrop::Pomegranate => domain::Crop::Pomegranate,
            AlwaCrop::Okra => domain::Crop::Okra,
            AlwaCrop::Eggplant => domain::Crop::Eggplant,
            AlwaCrop::Pepper => domain::Crop::Pepper,
            AlwaCrop::Apple => domain::Crop::Apple,
        }
    }
}

impl From<domain::Crop> for AlwaCrop {
    fn from(value: domain::Crop) -> Self {
        match value {
            domain::Crop::Wheat => AlwaCrop::Wheat,
            domain::Crop::Barley => AlwaCrop::Barley,
            domain::Crop::Tomato => AlwaCrop::Tomato,
            domain::Crop::Cucumber => AlwaCrop::Cucumber,
            domain::Crop::Potato => AlwaCrop::Potato,
            domain::Crop::Onion => AlwaCrop::Onion,
            domain::Crop::Watermelon => AlwaCrop::Watermelon,
            domain::Crop::Grape => AlwaCrop::Grape,
            domain::Crop::Olive => AlwaCrop::Olive,
            domain::Crop::Sunflower => AlwaCrop::Sunflower,
            domain::Crop::Chickpea => AlwaCrop::Chickpea,
            domain::Crop::Pomegranate => AlwaCrop::Pomegranate,
            domain::Crop::Okra => AlwaCrop::Okra,
            domain::Crop::Eggplant => AlwaCrop::Eggplant,
            domain::Crop::Pepper => AlwaCrop::Pepper,
            domain::Crop::Apple => AlwaCrop::Apple,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum AlwaGrade {
    A,
    B,
    C,
}

impl From<AlwaGrade> for domain::Grade {
    fn from(value: AlwaGrade) -> Self {
        match value {
            AlwaGrade::A => domain::Grade::A,
            AlwaGrade::B => domain::Grade::B,
            AlwaGrade::C => domain::Grade::C,
        }
    }
}

impl From<domain::Grade> for AlwaGrade {
    fn from(value: domain::Grade) -> Self {
        match value {
            domain::Grade::A => AlwaGrade::A,
            domain::Grade::B => AlwaGrade::B,
            domain::Grade::C => AlwaGrade::C,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum AlwaPickup {
    Farm,
    Alwa,
}

impl From<AlwaPickup> for domain::Pickup {
    fn from(value: AlwaPickup) -> Self {
        match value {
            AlwaPickup::Farm => domain::Pickup::Farm,
            AlwaPickup::Alwa => domain::Pickup::Alwa,
        }
    }
}

impl From<domain::Pickup> for AlwaPickup {
    fn from(value: domain::Pickup) -> Self {
        match value {
            domain::Pickup::Farm => AlwaPickup::Farm,
            domain::Pickup::Alwa => AlwaPickup::Alwa,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum AlwaListingStatus {
    Open,
    Sold,
    Closed,
    Cancelled,
}

impl From<AlwaListingStatus> for domain::ListingStatus {
    fn from(value: AlwaListingStatus) -> Self {
        match value {
            AlwaListingStatus::Open => domain::ListingStatus::Open,
            AlwaListingStatus::Sold => domain::ListingStatus::Sold,
            AlwaListingStatus::Closed => domain::ListingStatus::Closed,
            AlwaListingStatus::Cancelled => domain::ListingStatus::Cancelled,
        }
    }
}

impl From<domain::ListingStatus> for AlwaListingStatus {
    fn from(value: domain::ListingStatus) -> Self {
        match value {
            domain::ListingStatus::Open => AlwaListingStatus::Open,
            domain::ListingStatus::Sold => AlwaListingStatus::Sold,
            domain::ListingStatus::Closed => AlwaListingStatus::Closed,
            domain::ListingStatus::Cancelled => AlwaListingStatus::Cancelled,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum AlwaOfferStatus {
    Open,
    Accepted,
    Declined,
    Withdrawn,
}

impl From<AlwaOfferStatus> for domain::OfferStatus {
    fn from(value: AlwaOfferStatus) -> Self {
        match value {
            AlwaOfferStatus::Open => domain::OfferStatus::Open,
            AlwaOfferStatus::Accepted => domain::OfferStatus::Accepted,
            AlwaOfferStatus::Declined => domain::OfferStatus::Declined,
            AlwaOfferStatus::Withdrawn => domain::OfferStatus::Withdrawn,
        }
    }
}

impl From<domain::OfferStatus> for AlwaOfferStatus {
    fn from(value: domain::OfferStatus) -> Self {
        match value {
            domain::OfferStatus::Open => AlwaOfferStatus::Open,
            domain::OfferStatus::Accepted => AlwaOfferStatus::Accepted,
            domain::OfferStatus::Declined => AlwaOfferStatus::Declined,
            domain::OfferStatus::Withdrawn => AlwaOfferStatus::Withdrawn,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum AlwaBuyerKind {
    Shop,
    Restaurant,
    Trader,
    Other,
}

impl From<AlwaBuyerKind> for domain::BuyerKind {
    fn from(value: AlwaBuyerKind) -> Self {
        match value {
            AlwaBuyerKind::Shop => domain::BuyerKind::Shop,
            AlwaBuyerKind::Restaurant => domain::BuyerKind::Restaurant,
            AlwaBuyerKind::Trader => domain::BuyerKind::Trader,
            AlwaBuyerKind::Other => domain::BuyerKind::Other,
        }
    }
}

impl From<domain::BuyerKind> for AlwaBuyerKind {
    fn from(value: domain::BuyerKind) -> Self {
        match value {
            domain::BuyerKind::Shop => AlwaBuyerKind::Shop,
            domain::BuyerKind::Restaurant => AlwaBuyerKind::Restaurant,
            domain::BuyerKind::Trader => AlwaBuyerKind::Trader,
            domain::BuyerKind::Other => AlwaBuyerKind::Other,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum AlwaFairPrice {
    Fair,
    High,
    Low,
    Unknown,
}

impl From<AlwaFairPrice> for domain::FairPrice {
    fn from(value: AlwaFairPrice) -> Self {
        match value {
            AlwaFairPrice::Fair => domain::FairPrice::Fair,
            AlwaFairPrice::High => domain::FairPrice::High,
            AlwaFairPrice::Low => domain::FairPrice::Low,
            AlwaFairPrice::Unknown => domain::FairPrice::Unknown,
        }
    }
}

impl From<domain::FairPrice> for AlwaFairPrice {
    fn from(value: domain::FairPrice) -> Self {
        match value {
            domain::FairPrice::Fair => AlwaFairPrice::Fair,
            domain::FairPrice::High => AlwaFairPrice::High,
            domain::FairPrice::Low => AlwaFairPrice::Low,
            domain::FairPrice::Unknown => AlwaFairPrice::Unknown,
        }
    }
}

/// A day in a path or a query, `YYYY-MM-DD`.
pub fn parse_day(raw: &str) -> Result<NaiveDate, AppError> {
    raw.parse()
        .map_err(|_| DomainError::InvalidValue(format!("Day must be YYYY-MM-DD, got {raw}")).into())
}

/// A filter sent empty, as in `?market=&crop=`, is a filter not set.
fn given(value: Option<String>) -> Option<String> {
    value.filter(|value| !value.is_empty())
}

fn missing_id(what: &str) -> AppError {
    AppError::GlobalAppError(GlobalAppError::MissingValue(format!(
        "{what} is missing its id"
    )))
}

#[derive(Serialize, Deserialize, Debug, Clone, ToSchema)]
pub struct AlwaMarketResponse {
    pub slug: String,
    pub name_en: String,
    pub name_ku: String,
}

impl From<&Market> for AlwaMarketResponse {
    fn from(market: &Market) -> Self {
        Self {
            slug: market.slug().into(),
            name_en: market.name_en().clone(),
            name_ku: market.name_ku().clone(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, ToSchema)]
pub struct AlwaMarketsResponse {
    pub markets: Vec<AlwaMarketResponse>,
}

#[derive(Deserialize, Debug, Clone, IntoParams)]
#[into_params(parameter_in = Query)]
pub struct AlwaPricesQuery {
    /// `YYYY-MM-DD`. Left out: the latest day the market has any price for.
    pub day: Option<String>,
}

impl AlwaPricesQuery {
    pub fn into_input(self, market: String) -> Result<ViewMarketPricesInput, AppError> {
        Ok(ViewMarketPricesInput {
            market: MarketSlug::new(market)?,
            day: given(self.day).as_deref().map(parse_day).transpose()?,
        })
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, ToSchema)]
pub struct AlwaPriceResponse {
    pub crop: AlwaCrop,
    pub price_iqd_per_kg: i32,
    /// A government-set price, as for wheat.
    pub fixed: bool,
    /// Whole percent against the price 7 days earlier. `null` when there was
    /// none, or when the price is fixed.
    pub change_pct_7d: Option<i32>,
}

impl From<&PriceOnBoard> for AlwaPriceResponse {
    fn from(row: &PriceOnBoard) -> Self {
        Self {
            crop: (*row.price.crop()).into(),
            price_iqd_per_kg: row.price.price().value(),
            fixed: *row.price.fixed(),
            change_pct_7d: row.change_pct_7d,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, ToSchema)]
pub struct AlwaMarketPricesResponse {
    pub market: String,
    /// `null` when the market has no price at all yet.
    pub day: Option<NaiveDate>,
    pub prices: Vec<AlwaPriceResponse>,
}

impl From<&MarketPrices> for AlwaMarketPricesResponse {
    fn from(board: &MarketPrices) -> Self {
        Self {
            market: board.market.slug().into(),
            day: board.day,
            prices: board.prices.iter().map(Into::into).collect(),
        }
    }
}

#[derive(Deserialize, Debug, Clone, Copy, IntoParams)]
#[into_params(parameter_in = Query)]
pub struct AlwaHistoryQuery {
    /// How many days back, today included: 1 to 90. Left out: 7.
    pub days: Option<u32>,
}

impl AlwaHistoryQuery {
    pub fn into_input(self, market: String, crop: &str) -> Result<ViewPriceHistoryInput, AppError> {
        Ok(ViewPriceHistoryInput {
            market: MarketSlug::new(market)?,
            crop: domain::Crop::try_from(crop)?,
            days: match self.days {
                Some(days) => HistoryDays::new(days)?,
                None => HistoryDays::DEFAULT,
            },
        })
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, ToSchema)]
pub struct AlwaPricePointResponse {
    pub day: NaiveDate,
    pub price_iqd_per_kg: i32,
}

impl From<&Price> for AlwaPricePointResponse {
    fn from(price: &Price) -> Self {
        Self {
            day: *price.day(),
            price_iqd_per_kg: price.price().value(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, ToSchema)]
pub struct AlwaPriceHistoryResponse {
    pub market: String,
    pub crop: AlwaCrop,
    /// Oldest first. A day without a price is left out.
    pub history: Vec<AlwaPricePointResponse>,
}

#[derive(Serialize, Deserialize, Validate, Debug, Clone, ToSchema)]
pub struct RecordAlwaPriceParams {
    pub price_iqd_per_kg: i64,
    /// A government-set price, as for wheat.
    #[serde(default)]
    pub fixed: bool,
    /// Where the job read the price, 1 to 100 characters.
    pub source: String,
}

impl RecordAlwaPriceParams {
    pub fn into_input(
        self,
        market: String,
        crop: &str,
        day: &str,
    ) -> Result<RecordPriceInput, AppError> {
        Ok(RecordPriceInput {
            market: MarketSlug::new(market)?,
            crop: domain::Crop::try_from(crop)?,
            day: parse_day(day)?,
            price: PricePerKg::new(self.price_iqd_per_kg)?,
            fixed: self.fixed,
            source: PriceSource::new(self.source)?,
        })
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, ToSchema)]
pub struct AlwaRecordedPriceResponse {
    pub market: String,
    pub crop: AlwaCrop,
    pub day: NaiveDate,
    pub price_iqd_per_kg: i32,
    pub fixed: bool,
    pub source: String,
    pub updated_at: DateTime<Utc>,
}

#[derive(Serialize, Deserialize, Debug, Clone, ToSchema)]
pub struct AlwaOnePriceResponse {
    pub price: AlwaRecordedPriceResponse,
}

impl From<(&Market, &Price)> for AlwaOnePriceResponse {
    fn from((market, price): (&Market, &Price)) -> Self {
        Self {
            price: AlwaRecordedPriceResponse {
                market: market.slug().into(),
                crop: (*price.crop()).into(),
                day: *price.day(),
                price_iqd_per_kg: price.price().value(),
                fixed: *price.fixed(),
                source: price.source().into(),
                updated_at: *price.updated_at(),
            },
        }
    }
}

#[derive(Deserialize, Debug, Clone, IntoParams)]
#[into_params(parameter_in = Query)]
pub struct AlwaListingsQuery {
    /// Market slug, for example `sulaymaniyah`.
    pub market: Option<String>,
    /// Crop code, for example `tomato`.
    pub crop: Option<String>,
    /// `open` (the default), `sold`, `closed` or `cancelled`.
    pub status: Option<String>,
}

impl AlwaListingsQuery {
    pub fn into_input(
        self,
        pagination: crate::app::Pagination,
    ) -> Result<BrowseListingsInput, AppError> {
        Ok(BrowseListingsInput {
            market: given(self.market).map(MarketSlug::new).transpose()?,
            crop: given(self.crop)
                .as_deref()
                .map(domain::Crop::try_from)
                .transpose()?,
            status: given(self.status)
                .as_deref()
                .map(domain::ListingStatus::try_from)
                .transpose()?
                .unwrap_or(domain::ListingStatus::Open),
            pagination,
        })
    }
}

/// A listing on the public board. The id is an opaque string to the app. The
/// seller's phone is never part of it.
#[derive(Serialize, Deserialize, Debug, Clone, ToSchema)]
pub struct AlwaListingSummaryResponse {
    pub id: String,
    pub crop: AlwaCrop,
    pub quantity_kg: i32,
    pub asking_price_iqd_per_kg: i32,
    pub grade: Option<AlwaGrade>,
    pub pickup: AlwaPickup,
    /// Market slug.
    pub market: String,
    pub zone_slug: Option<String>,
    pub seller_name: Option<String>,
    pub closes_at: DateTime<Utc>,
    /// `closed` once `closes_at` has passed with no deal.
    pub status: AlwaListingStatus,
    /// How many open offers the listing has.
    pub offers: usize,
    /// The highest open offer, `null` when there is none.
    pub best_offer_iqd_per_kg: Option<i32>,
    /// The asking price against the alwa's own price for the crop.
    pub fair_price: AlwaFairPrice,
}

impl TryFrom<&ListingCard> for AlwaListingSummaryResponse {
    type Error = AppError;

    fn try_from(card: &ListingCard) -> Result<Self, Self::Error> {
        let listing = card.listing();

        Ok(Self {
            id: listing_id(listing)?,
            crop: (*listing.crop()).into(),
            quantity_kg: listing.quantity().value(),
            asking_price_iqd_per_kg: listing.asking_price().value(),
            grade: listing.grade().map(Into::into),
            pickup: (*listing.pickup()).into(),
            market: listing.market().into(),
            zone_slug: listing.zone_slug().as_ref().map(Into::into),
            seller_name: listing.seller_name().as_ref().map(Into::into),
            closes_at: *listing.closes_at(),
            status: (*card.status()).into(),
            offers: card.open_offers(),
            best_offer_iqd_per_kg: card.best_offer().map(|price| price.value()),
            fair_price: (*card.fair_price()).into(),
        })
    }
}

fn listing_id(listing: &Listing) -> Result<String, AppError> {
    Ok(listing
        .id()
        .ok_or_else(|| missing_id("Listing"))?
        .to_string())
}

#[derive(Serialize, Deserialize, Debug, Clone, ToSchema)]
pub struct AlwaListingsResponse {
    pub listings: Vec<AlwaListingSummaryResponse>,
    /// How many listings match in all, on every page.
    pub count: u64,
    pub page: u64,
    pub rows_per_page: u64,
}

#[derive(Serialize, Deserialize, Validate, Debug, Clone, ToSchema)]
pub struct MakeAlwaOfferParams {
    /// Shown to the seller and on the board, 1 to 60 characters.
    pub buyer_name: String,
    pub buyer_kind: AlwaBuyerKind,
    /// 1 kg up to the quantity on sale.
    pub quantity_kg: i64,
    pub price_iqd_per_kg: i64,
}

impl MakeAlwaOfferParams {
    pub fn into_input(self) -> Result<OfferDraft, AppError> {
        Ok(OfferDraft {
            buyer_name: DisplayName::new(self.buyer_name)?,
            buyer_kind: self.buyer_kind.into(),
            quantity: QuantityKg::new(self.quantity_kg)?,
            price: PricePerKg::new(self.price_iqd_per_kg)?,
        })
    }
}

/// An offer as anyone may see it, as on the market floor. `buyer_phone` is
/// there only for the seller, and only on the offer they accepted.
#[derive(Serialize, Deserialize, Debug, Clone, ToSchema)]
pub struct AlwaOfferResponse {
    pub id: String,
    pub buyer_name: String,
    pub buyer_kind: AlwaBuyerKind,
    pub quantity_kg: i32,
    pub price_iqd_per_kg: i32,
    pub status: AlwaOfferStatus,
    pub created_at: DateTime<Utc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub buyer_phone: Option<String>,
}

impl AlwaOfferResponse {
    fn new(offer: &Offer, buyer_phone: Option<&Phone>) -> Result<Self, AppError> {
        Ok(Self {
            id: offer.id().ok_or_else(|| missing_id("Offer"))?.to_string(),
            buyer_name: offer.buyer_name().into(),
            buyer_kind: (*offer.buyer_kind()).into(),
            quantity_kg: offer.quantity().value(),
            price_iqd_per_kg: offer.price().value(),
            status: (*offer.status()).into(),
            created_at: *offer.created_at(),
            buyer_phone: buyer_phone.map(Into::into),
        })
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, ToSchema)]
pub struct AlwaOneOfferResponse {
    pub offer: AlwaOfferResponse,
}

impl TryFrom<&Offer> for AlwaOneOfferResponse {
    type Error = AppError;

    fn try_from(offer: &Offer) -> Result<Self, Self::Error> {
        Ok(Self {
            offer: AlwaOfferResponse::new(offer, None)?,
        })
    }
}

#[derive(Serialize, Deserialize, Validate, Debug, Clone, ToSchema)]
pub struct PostAlwaListingParams {
    pub crop: AlwaCrop,
    /// 1 to 1,000,000 kg.
    pub quantity_kg: i64,
    pub asking_price_iqd_per_kg: i64,
    pub grade: Option<AlwaGrade>,
    pub pickup: AlwaPickup,
    /// Market slug, for example `sulaymaniyah`.
    pub market: String,
    /// Where the crop comes from: lower-case letters and hyphens.
    pub zone_slug: Option<String>,
    /// Up to 200 characters.
    pub note: Option<String>,
    /// In the future, at most 14 days ahead.
    pub closes_at: DateTime<Utc>,
    /// Shown to buyers instead of the phone, 1 to 60 characters.
    pub seller_name: Option<String>,
}

impl PostAlwaListingParams {
    pub fn into_input(self, idempotency_key: Option<String>) -> Result<PostListingInput, AppError> {
        Ok(PostListingInput {
            idempotency_key: idempotency_key.map(IdempotencyKey::new).transpose()?,
            market: MarketSlug::new(self.market)?,
            draft: ListingDraft {
                seller_name: self.seller_name.map(DisplayName::new).transpose()?,
                crop: self.crop.into(),
                quantity: QuantityKg::new(self.quantity_kg)?,
                asking_price: PricePerKg::new(self.asking_price_iqd_per_kg)?,
                grade: self.grade.map(Into::into),
                pickup: self.pickup.into(),
                zone_slug: self.zone_slug.map(ZoneSlug::new).transpose()?,
                note: self.note.map(Note::new).transpose()?,
                closes_at: self.closes_at,
            },
        })
    }
}

/// One listing in full. Here `offers` is the list of offers, highest price
/// first, where the board shows only how many are open.
#[derive(Serialize, Deserialize, Debug, Clone, ToSchema)]
pub struct AlwaListingResponse {
    pub id: String,
    pub crop: AlwaCrop,
    pub quantity_kg: i32,
    pub asking_price_iqd_per_kg: i32,
    pub grade: Option<AlwaGrade>,
    pub pickup: AlwaPickup,
    /// Market slug.
    pub market: String,
    pub zone_slug: Option<String>,
    pub seller_name: Option<String>,
    pub closes_at: DateTime<Utc>,
    /// `closed` once `closes_at` has passed with no deal.
    pub status: AlwaListingStatus,
    pub best_offer_iqd_per_kg: Option<i32>,
    pub fair_price: AlwaFairPrice,
    pub note: Option<String>,
    pub created_at: DateTime<Utc>,
    pub offers: Vec<AlwaOfferResponse>,
}

impl AlwaListingResponse {
    /// `viewer` is the signed-in phone, or `None` on the public routes. It
    /// decides only whether the accepted buyer's phone is shown.
    pub fn new(card: &ListingCard, viewer: Option<&Phone>) -> Result<Self, AppError> {
        let listing = card.listing();
        let summary = AlwaListingSummaryResponse::try_from(card)?;

        let offers = card
            .offers()
            .iter()
            .map(|offer| {
                let buyer_phone =
                    viewer.and_then(|viewer| offer.buyer_phone_shown_to(viewer, listing));

                AlwaOfferResponse::new(offer, buyer_phone)
            })
            .collect::<Result<Vec<_>, _>>()?;

        Ok(Self {
            id: summary.id,
            crop: summary.crop,
            quantity_kg: summary.quantity_kg,
            asking_price_iqd_per_kg: summary.asking_price_iqd_per_kg,
            grade: summary.grade,
            pickup: summary.pickup,
            market: summary.market,
            zone_slug: summary.zone_slug,
            seller_name: summary.seller_name,
            closes_at: summary.closes_at,
            status: summary.status,
            best_offer_iqd_per_kg: summary.best_offer_iqd_per_kg,
            fair_price: summary.fair_price,
            note: listing.note().as_ref().map(Into::into),
            created_at: *listing.created_at(),
            offers,
        })
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, ToSchema)]
pub struct AlwaOneListingResponse {
    pub listing: AlwaListingResponse,
}

impl AlwaOneListingResponse {
    pub fn new(card: &ListingCard, viewer: Option<&Phone>) -> Result<Self, AppError> {
        Ok(Self {
            listing: AlwaListingResponse::new(card, viewer)?,
        })
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, ToSchema)]
pub struct AlwaMyListingsResponse {
    pub listings: Vec<AlwaListingResponse>,
}

impl AlwaMyListingsResponse {
    pub fn new(cards: &[ListingCard], viewer: &Phone) -> Result<Self, AppError> {
        Ok(Self {
            listings: cards
                .iter()
                .map(|card| AlwaListingResponse::new(card, Some(viewer)))
                .collect::<Result<Vec<_>, _>>()?,
        })
    }
}

/// The listing an offer was made on, in short.
#[derive(Serialize, Deserialize, Debug, Clone, ToSchema)]
pub struct AlwaOfferListingResponse {
    pub id: String,
    pub crop: AlwaCrop,
    pub quantity_kg: i32,
    pub asking_price_iqd_per_kg: i32,
    /// Market slug.
    pub market: String,
    pub seller_name: Option<String>,
    pub closes_at: DateTime<Utc>,
    pub status: AlwaListingStatus,
}

/// One of the signed-in buyer's own offers. `seller_phone` is there only
/// once the offer was accepted.
#[derive(Serialize, Deserialize, Debug, Clone, ToSchema)]
pub struct AlwaMyOfferResponse {
    pub id: String,
    pub buyer_name: String,
    pub buyer_kind: AlwaBuyerKind,
    pub quantity_kg: i32,
    pub price_iqd_per_kg: i32,
    pub status: AlwaOfferStatus,
    pub created_at: DateTime<Utc>,
    pub listing: AlwaOfferListingResponse,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub seller_phone: Option<String>,
}

impl AlwaMyOfferResponse {
    fn new(placed: &PlacedOffer, viewer: &Phone) -> Result<Self, AppError> {
        let offer = AlwaOfferResponse::new(placed.offer(), None)?;
        let listing = placed.listing();

        Ok(Self {
            id: offer.id,
            buyer_name: offer.buyer_name,
            buyer_kind: offer.buyer_kind,
            quantity_kg: offer.quantity_kg,
            price_iqd_per_kg: offer.price_iqd_per_kg,
            status: offer.status,
            created_at: offer.created_at,
            listing: AlwaOfferListingResponse {
                id: listing_id(listing)?,
                crop: (*listing.crop()).into(),
                quantity_kg: listing.quantity().value(),
                asking_price_iqd_per_kg: listing.asking_price().value(),
                market: listing.market().into(),
                seller_name: listing.seller_name().as_ref().map(Into::into),
                closes_at: *listing.closes_at(),
                status: (*placed.listing_status()).into(),
            },
            seller_phone: listing
                .seller_phone_shown_to(viewer, placed.offer())
                .map(Into::into),
        })
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, ToSchema)]
pub struct AlwaMyOffersResponse {
    pub offers: Vec<AlwaMyOfferResponse>,
}

impl AlwaMyOffersResponse {
    pub fn new(placed: &[PlacedOffer], viewer: &Phone) -> Result<Self, AppError> {
        Ok(Self {
            offers: placed
                .iter()
                .map(|placed| AlwaMyOfferResponse::new(placed, viewer))
                .collect::<Result<Vec<_>, _>>()?,
        })
    }
}

#[derive(Deserialize, Debug, Clone, IntoParams)]
#[into_params(parameter_in = Query)]
pub struct AlwaDealsQuery {
    /// Market slug. Left out: every market.
    pub market: Option<String>,
    /// `YYYY-MM-DD`. Left out: today (UTC).
    pub day: Option<String>,
}

impl AlwaDealsQuery {
    pub fn into_input(self) -> Result<ListDealsInput, AppError> {
        Ok(ListDealsInput {
            market: given(self.market).map(MarketSlug::new).transpose()?,
            day: given(self.day).as_deref().map(parse_day).transpose()?,
        })
    }
}

/// A sale, for the dashboard. Names only: a deal shows no phone.
#[derive(Serialize, Deserialize, Debug, Clone, ToSchema)]
pub struct AlwaDealResponse {
    pub listing_id: String,
    pub crop: AlwaCrop,
    /// The quantity that was sold, which is the accepted offer's.
    pub quantity_kg: i32,
    pub asking_price_iqd_per_kg: i32,
    pub sold_price_iqd_per_kg: i32,
    /// Whole percent the sold price is above (or, negative, below) the
    /// asking price.
    pub vs_asking_pct: i32,
    pub seller_name: Option<String>,
    pub buyer_name: String,
    pub zone_slug: Option<String>,
    pub accepted_at: DateTime<Utc>,
}

impl TryFrom<&Deal> for AlwaDealResponse {
    type Error = AppError;

    fn try_from(deal: &Deal) -> Result<Self, Self::Error> {
        let listing = deal.listing();
        let offer = deal.offer();

        Ok(Self {
            listing_id: listing_id(listing)?,
            crop: (*listing.crop()).into(),
            quantity_kg: offer.quantity().value(),
            asking_price_iqd_per_kg: listing.asking_price().value(),
            sold_price_iqd_per_kg: offer.price().value(),
            vs_asking_pct: deal.vs_asking_pct(),
            seller_name: listing.seller_name().as_ref().map(Into::into),
            buyer_name: offer.buyer_name().into(),
            zone_slug: listing.zone_slug().as_ref().map(Into::into),
            accepted_at: deal.accepted_at(),
        })
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, ToSchema)]
pub struct AlwaDealsSummaryResponse {
    pub deals: usize,
    pub tonnes: f64,
}

#[derive(Serialize, Deserialize, Debug, Clone, ToSchema)]
pub struct AlwaDealsResponse {
    pub day: NaiveDate,
    pub deals: Vec<AlwaDealResponse>,
    pub summary: AlwaDealsSummaryResponse,
}

impl TryFrom<(NaiveDate, &[Deal])> for AlwaDealsResponse {
    type Error = AppError;

    fn try_from((day, deals): (NaiveDate, &[Deal])) -> Result<Self, Self::Error> {
        let summary = DealsSummary::of(deals);

        Ok(Self {
            day,
            deals: deals
                .iter()
                .map(AlwaDealResponse::try_from)
                .collect::<Result<Vec<_>, _>>()?,
            summary: AlwaDealsSummaryResponse {
                deals: summary.deals,
                tonnes: summary.tonnes,
            },
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::features::alwa::app::testing::{
        BUYER, OTHER_BUYER, SELLER, a_sold_listing, an_open_listing, an_open_offer, phone,
    };

    fn json<T: Serialize>(value: &T) -> serde_json::Value {
        serde_json::to_value(value).expect("serializes")
    }

    fn a_sold_card() -> ListingCard {
        let (listing, offers) = a_sold_listing(7);

        ListingCard::assemble(listing, &offers, &[], Utc::now())
    }

    #[test]
    fn a_public_listing_shows_no_phone_at_all() {
        let open = an_open_listing(7);
        let offers = [an_open_offer(1, &open, BUYER, 950)];
        let open_card = ListingCard::assemble(open, &offers, &[], Utc::now());

        for card in [open_card, a_sold_card()] {
            let body = json(&AlwaOneListingResponse::new(&card, None).expect("body")).to_string();

            assert!(!body.contains("phone"), "{body}");
            assert!(!body.contains("+964"), "{body}");
        }
    }

    #[test]
    fn the_board_summary_has_no_phone_and_counts_offers() {
        let open = an_open_listing(7);
        let offers = [an_open_offer(1, &open, BUYER, 950)];
        let card = ListingCard::assemble(open, &offers, &[], Utc::now());

        let body = json(&AlwaListingSummaryResponse::try_from(&card).expect("body"));

        assert_eq!(body["id"], "7");
        assert_eq!(body["offers"], 1);
        assert_eq!(body["best_offer_iqd_per_kg"], 950);
        assert_eq!(body["fair_price"], "unknown");
        assert_eq!(body["market"], "sulaymaniyah");
        assert!(!body.to_string().contains("+964"));
    }

    #[test]
    fn the_seller_sees_the_phone_of_the_accepted_buyer_only() {
        let body =
            json(&AlwaOneListingResponse::new(&a_sold_card(), Some(&phone(SELLER))).expect("body"));
        let offers = body["listing"]["offers"].as_array().expect("offers");

        assert_eq!(offers[0]["status"], "accepted");
        assert_eq!(offers[0]["buyer_phone"], BUYER);
        assert_eq!(offers[1]["status"], "declined");
        assert!(
            offers[1].get("buyer_phone").is_none(),
            "a declined buyer's phone stays private"
        );
        assert!(body["listing"].get("seller_phone").is_none());
    }

    #[test]
    fn a_stranger_with_a_token_sees_no_phone_on_a_sold_listing() {
        let body = json(
            &AlwaOneListingResponse::new(&a_sold_card(), Some(&phone(OTHER_BUYER))).expect("body"),
        );

        assert!(!body.to_string().contains("+964"));
    }

    #[test]
    fn the_accepted_buyer_sees_the_sellers_phone_and_the_declined_one_does_not() {
        let (listing, offers) = a_sold_listing(7);
        let placed = |index: usize| {
            vec![PlacedOffer::new(
                offers[index].clone(),
                listing.clone(),
                Utc::now(),
            )]
        };

        let accepted = json(&AlwaMyOffersResponse::new(&placed(0), &phone(BUYER)).expect("body"));
        let declined =
            json(&AlwaMyOffersResponse::new(&placed(1), &phone(OTHER_BUYER)).expect("body"));

        assert_eq!(accepted["offers"][0]["seller_phone"], SELLER);
        assert_eq!(accepted["offers"][0]["listing"]["status"], "sold");
        assert!(declined["offers"][0].get("seller_phone").is_none());
        assert!(!declined.to_string().contains(SELLER));
    }

    #[test]
    fn a_new_offer_answer_does_not_echo_the_buyers_phone() {
        let listing = an_open_listing(7);
        let offer = an_open_offer(1, &listing, BUYER, 950);

        let body = json(&AlwaOneOfferResponse::try_from(&offer).expect("body"));

        assert_eq!(body["offer"]["id"], "1");
        assert!(!body.to_string().contains("+964"));
    }

    #[test]
    fn a_deal_on_the_dashboard_shows_names_not_phones() {
        let (listing, offers) = a_sold_listing(7);
        let deal = Deal::new(listing, offers[0].clone()).expect("deal");
        let day = deal.accepted_at().date_naive();

        let body = json(&AlwaDealsResponse::try_from((day, [deal].as_slice())).expect("body"));

        assert_eq!(body["deals"][0]["listing_id"], "7");
        assert_eq!(body["deals"][0]["sold_price_iqd_per_kg"], 950);
        assert_eq!(body["deals"][0]["vs_asking_pct"], -5);
        assert_eq!(body["summary"]["deals"], 1);
        assert_eq!(body["summary"]["tonnes"], 0.5);
        assert!(!body.to_string().contains("+964"));
    }

    #[test]
    fn an_empty_filter_is_a_filter_not_set() {
        let input = AlwaListingsQuery {
            market: Some(String::new()),
            crop: Some(String::new()),
            status: None,
        }
        .into_input(crate::app::Pagination::new(1, 20))
        .expect("input");

        assert!(input.market.is_none());
        assert!(input.crop.is_none());
        assert_eq!(input.status, domain::ListingStatus::Open);
    }

    #[test]
    fn an_unknown_crop_status_or_day_in_a_query_is_refused() {
        let query = |crop: Option<&str>, status: Option<&str>| AlwaListingsQuery {
            market: None,
            crop: crop.map(str::to_string),
            status: status.map(str::to_string),
        };
        let pagination = crate::app::Pagination::new(1, 20);

        assert!(query(Some("empty"), None).into_input(pagination).is_err());
        assert!(query(None, Some("gone")).into_input(pagination).is_err());
        assert!(parse_day("2026-10-08").is_ok());
        assert!(parse_day("08/10/2026").is_err());
        assert!(parse_day("2026-02-30").is_err());
    }
}
