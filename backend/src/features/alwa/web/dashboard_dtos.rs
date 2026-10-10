//! What the Ministry dashboard sends and reads. Unlike the public shapes in
//! `dtos.rs`, a listing here carries the seller's phone and an offer the
//! buyer's: staff holding `alwa:read` moderate with them.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};
use validator::Validate;

use super::dtos::{
    AlwaBuyerKind, AlwaFairPrice, AlwaGrade, AlwaListingStatus, AlwaMarketResponse,
    AlwaOfferStatus, AlwaOnePriceResponse, AlwaPickup, AlwaProductGroup, AlwaRecordedPriceResponse,
    AlwaUnit, KgFields, RecordAlwaPriceParams, given, listing_id, missing_id, parse_day,
};

use crate::{
    app::Pagination,
    features::alwa::{
        app::{
            AppError,
            use_cases::{
                CreateMarketInput, ListAllListingsInput, ListStoredPricesInput,
                ModerateListingInput, RecordPriceInput,
            },
        },
        domain::{
            self, GeoPoint, ListingCard, Market, MarketName, MarketNames, MarketSlug, Note, Offer,
            Price,
        },
    },
    shared::Phone,
};

#[derive(Serialize, Deserialize, Validate, Debug, Clone, ToSchema)]
pub struct CreateAlwaMarketParams {
    /// 1 to 40 lower-case letters and hyphens. It names the market in every
    /// URL and cannot be changed later.
    pub slug: String,
    /// 1 to 80 characters.
    pub name_en: String,
    /// 1 to 80 characters.
    pub name_ku: String,
    /// Where the alwa is, WGS84, inside the Kurdistan Region. Given
    /// together with `lon` or not at all.
    pub lat: Option<f64>,
    pub lon: Option<f64>,
}

impl CreateAlwaMarketParams {
    pub fn into_input(self) -> Result<CreateMarketInput, AppError> {
        Ok(CreateMarketInput {
            slug: MarketSlug::new(self.slug)?,
            names: MarketNames {
                name_en: MarketName::new(self.name_en)?,
                name_ku: MarketName::new(self.name_ku)?,
            },
            point: GeoPoint::from_pair(self.lat, self.lon, GeoPoint::in_region)?,
        })
    }
}

#[derive(Serialize, Deserialize, Validate, Debug, Clone, ToSchema)]
pub struct UpdateAlwaMarketParams {
    /// 1 to 80 characters.
    pub name_en: String,
    /// 1 to 80 characters.
    pub name_ku: String,
    /// Where the alwa is, WGS84, inside the Kurdistan Region. Given
    /// together with `lon`; both left out, the place stays as it was.
    pub lat: Option<f64>,
    pub lon: Option<f64>,
}

impl UpdateAlwaMarketParams {
    pub fn into_input(
        self,
        slug: String,
    ) -> Result<(MarketSlug, MarketNames, Option<GeoPoint>), AppError> {
        Ok((
            MarketSlug::new(slug)?,
            MarketNames {
                name_en: MarketName::new(self.name_en)?,
                name_ku: MarketName::new(self.name_ku)?,
            },
            GeoPoint::from_pair(self.lat, self.lon, GeoPoint::in_region)?,
        ))
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, ToSchema)]
pub struct AlwaOneMarketResponse {
    pub market: AlwaMarketResponse,
}

impl From<&Market> for AlwaOneMarketResponse {
    fn from(market: &Market) -> Self {
        Self {
            market: market.into(),
        }
    }
}

#[derive(Deserialize, Debug, Clone, IntoParams)]
#[into_params(parameter_in = Query)]
pub struct AlwaStoredPricesQuery {
    /// Crop code, for example `tomato`.
    pub crop: Option<String>,
    /// First day, included, `YYYY-MM-DD`.
    pub from: Option<String>,
    /// Last day, included, `YYYY-MM-DD`.
    pub to: Option<String>,
}

impl AlwaStoredPricesQuery {
    pub fn into_input(
        self,
        market: String,
        pagination: Pagination,
    ) -> Result<ListStoredPricesInput, AppError> {
        Ok(ListStoredPricesInput {
            market: MarketSlug::new(market)?,
            crop: given(self.crop)
                .as_deref()
                .map(domain::Crop::new)
                .transpose()?,
            from: given(self.from).as_deref().map(parse_day).transpose()?,
            to: given(self.to).as_deref().map(parse_day).transpose()?,
            pagination,
        })
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, ToSchema)]
pub struct AlwaStoredPricesResponse {
    /// Newest day first.
    pub prices: Vec<AlwaRecordedPriceResponse>,
    /// How many prices match in all, on every page.
    pub count: u64,
    pub page: u64,
    pub rows_per_page: u64,
}

impl AlwaStoredPricesResponse {
    pub fn new(market: &Market, prices: &[Price], count: u64, pagination: &Pagination) -> Self {
        Self {
            prices: prices
                .iter()
                .map(|price| AlwaOnePriceResponse::from((market, price)).price)
                .collect(),
            count,
            page: *pagination.page(),
            rows_per_page: *pagination.rows_per_page(),
        }
    }
}

/// The body the price job sends, plus the crop and the day it has in its
/// path.
#[derive(Serialize, Deserialize, Validate, Debug, Clone, ToSchema)]
pub struct CreateAlwaPriceParams {
    /// Crop code, for example `tomato`.
    pub crop: String,
    /// `YYYY-MM-DD`.
    pub day: String,
    pub price_iqd_per_kg: i64,
    /// A government-set price, as for wheat.
    #[serde(default)]
    pub fixed: bool,
    /// Where the price was read, 1 to 100 characters.
    pub source: String,
}

impl CreateAlwaPriceParams {
    pub fn into_input(self, market: String) -> Result<RecordPriceInput, AppError> {
        RecordAlwaPriceParams {
            price_iqd_per_kg: self.price_iqd_per_kg,
            fixed: self.fixed,
            source: self.source,
        }
        .into_input(market, &self.crop, &self.day)
    }
}

#[derive(Deserialize, Debug, Clone, IntoParams)]
#[into_params(parameter_in = Query)]
pub struct AlwaModerationQuery {
    /// Market slug, for example `sulaymaniyah`.
    pub market: Option<String>,
    /// Crop code, for example `tomato`.
    pub crop: Option<String>,
    /// `open`, `sold`, `closed` or `cancelled`. Left out: every status.
    pub status: Option<String>,
    /// The seller's phone in E.164. Send the `+` as `%2B`.
    pub seller_phone: Option<String>,
}

impl AlwaModerationQuery {
    pub fn into_input(self, pagination: Pagination) -> Result<ListAllListingsInput, AppError> {
        Ok(ListAllListingsInput {
            market: given(self.market).map(MarketSlug::new).transpose()?,
            crop: given(self.crop)
                .as_deref()
                .map(domain::Crop::new)
                .transpose()?,
            status: given(self.status)
                .as_deref()
                .map(domain::ListingStatus::try_from)
                .transpose()?,
            seller: given(self.seller_phone).map(Phone::new).transpose()?,
            pagination,
        })
    }
}

/// A listing as staff see it, with the seller's phone.
#[derive(Serialize, Deserialize, Debug, Clone, ToSchema)]
pub struct AlwaModeratedListingResponse {
    pub id: String,
    pub seller_phone: String,
    pub seller_name: Option<String>,
    /// The product code, from `GET /v1/products`.
    pub product: String,
    pub group: AlwaProductGroup,
    /// What `quantity` counts and what `asking_price_iqd` is for one of,
    /// as the product had it when the listing was posted.
    pub unit: AlwaUnit,
    /// A whole number of `unit`.
    pub quantity: i32,
    /// For one `unit`.
    pub asking_price_iqd: i32,
    /// The same as `product`, `quantity` and `asking_price_iqd` when the
    /// unit is `kg`, for apps from before products had units; `null` for
    /// every other unit.
    pub crop: Option<String>,
    pub quantity_kg: Option<i32>,
    pub asking_price_iqd_per_kg: Option<i32>,
    pub grade: Option<AlwaGrade>,
    /// `null` when the seller did not say.
    pub pickup: Option<AlwaPickup>,
    /// Market slug, `null` for a listing at no alwa.
    pub market: Option<String>,
    pub zone_slug: Option<String>,
    /// Where the crop is, WGS84. `null` for a listing posted without a
    /// point.
    pub lat: Option<f64>,
    pub lon: Option<f64>,
    /// The seller's own note.
    pub note: Option<String>,
    pub closes_at: DateTime<Utc>,
    /// `closed` once `closes_at` has passed with no deal, or once staff
    /// closed it.
    pub status: AlwaListingStatus,
    /// How many open offers the listing has.
    pub open_offers: usize,
    pub fair_price: AlwaFairPrice,
    /// The staff member who closed it, `null` unless staff did.
    pub closed_by_staff_id: Option<String>,
    /// The reason they gave, if any.
    pub moderation_note: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl TryFrom<&ListingCard> for AlwaModeratedListingResponse {
    type Error = AppError;

    fn try_from(card: &ListingCard) -> Result<Self, Self::Error> {
        let listing = card.listing();
        let moderation = listing.moderation().as_ref();
        let kg = KgFields::from(listing);

        Ok(Self {
            id: listing_id(listing)?,
            seller_phone: listing.seller_phone().into(),
            seller_name: listing.seller_name().as_ref().map(Into::into),
            product: (*listing.crop()).into(),
            group: (*listing.group()).into(),
            unit: (*listing.unit()).into(),
            quantity: listing.quantity().value(),
            asking_price_iqd: listing.asking_price().value(),
            crop: kg.crop,
            quantity_kg: kg.quantity_kg,
            asking_price_iqd_per_kg: kg.asking_price_iqd_per_kg,
            grade: listing.grade().map(Into::into),
            pickup: listing.pickup().map(Into::into),
            market: listing.market().as_ref().map(Into::into),
            zone_slug: listing.zone_slug().as_ref().map(Into::into),
            lat: listing.point().map(|point| point.lat()),
            lon: listing.point().map(|point| point.lon()),
            note: listing.note().as_ref().map(Into::into),
            closes_at: *listing.closes_at(),
            status: (*card.status()).into(),
            open_offers: card.open_offers(),
            fair_price: (*card.fair_price()).into(),
            closed_by_staff_id: moderation.map(|moderation| moderation.staff_id().to_string()),
            moderation_note: moderation
                .and_then(|moderation| moderation.note().as_ref().map(Into::into)),
            created_at: *listing.created_at(),
            updated_at: *listing.updated_at(),
        })
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, ToSchema)]
pub struct AlwaModeratedListingsResponse {
    /// Newest first.
    pub listings: Vec<AlwaModeratedListingResponse>,
    /// How many listings match in all, on every page.
    pub count: u64,
    pub page: u64,
    pub rows_per_page: u64,
}

/// An offer as staff see it, with the buyer's phone.
#[derive(Serialize, Deserialize, Debug, Clone, ToSchema)]
pub struct AlwaModeratedOfferResponse {
    pub id: String,
    pub buyer_phone: String,
    pub buyer_name: String,
    pub buyer_kind: AlwaBuyerKind,
    pub quantity_kg: i32,
    pub price_iqd_per_kg: i32,
    pub status: AlwaOfferStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl TryFrom<&Offer> for AlwaModeratedOfferResponse {
    type Error = AppError;

    fn try_from(offer: &Offer) -> Result<Self, Self::Error> {
        Ok(Self {
            id: offer.id().ok_or_else(|| missing_id("Offer"))?.to_string(),
            buyer_phone: offer.buyer_phone().into(),
            buyer_name: offer.buyer_name().into(),
            buyer_kind: (*offer.buyer_kind()).into(),
            quantity_kg: offer.quantity().value(),
            price_iqd_per_kg: offer.price().value(),
            status: (*offer.status()).into(),
            created_at: *offer.created_at(),
            updated_at: *offer.updated_at(),
        })
    }
}

/// One listing with every offer on it, highest price first.
#[derive(Serialize, Deserialize, Debug, Clone, ToSchema)]
pub struct AlwaModeratedListingDetailResponse {
    #[serde(flatten)]
    pub listing: AlwaModeratedListingResponse,
    pub offers: Vec<AlwaModeratedOfferResponse>,
}

#[derive(Serialize, Deserialize, Debug, Clone, ToSchema)]
pub struct AlwaOneModeratedListingResponse {
    pub listing: AlwaModeratedListingDetailResponse,
}

impl TryFrom<&ListingCard> for AlwaOneModeratedListingResponse {
    type Error = AppError;

    fn try_from(card: &ListingCard) -> Result<Self, Self::Error> {
        Ok(Self {
            listing: AlwaModeratedListingDetailResponse {
                listing: card.try_into()?,
                offers: card
                    .offers()
                    .iter()
                    .map(AlwaModeratedOfferResponse::try_from)
                    .collect::<Result<Vec<_>, _>>()?,
            },
        })
    }
}

#[derive(Serialize, Deserialize, Validate, Debug, Clone, ToSchema)]
pub struct ModerateAlwaListingParams {
    /// Only `closed` is allowed: staff cannot sell or reopen a listing.
    pub status: AlwaListingStatus,
    /// Why, up to 200 characters. Repeat it unchanged when retrying.
    pub note: Option<String>,
}

impl ModerateAlwaListingParams {
    pub fn into_input(self, id: i32) -> Result<ModerateListingInput, AppError> {
        Ok(ModerateListingInput {
            id,
            status: self.status.into(),
            note: self.note.map(Note::new).transpose()?,
        })
    }
}
