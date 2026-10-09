use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use chrono::{DateTime, Duration, NaiveDate, Utc};

use crate::{
    app::{AuthContext, Pagination, User},
    features::alwa::{
        app::{
            AlwaRepository, AppError, CropDirectory, ListingFilter, ModerationFilter,
            StoredPriceFilter, ZoneLocator,
        },
        domain::{
            ActiveCrops, AlwaError, BuyerKind, Crop, Deal, DisplayName, GeoPoint, Grade,
            IdempotencyKey, Listing, ListingDraft, ListingStatus, Market, MarketName, MarketNames,
            MarketSlug, Offer, OfferDraft, OfferStatus, Pickup, Price, PricePerKg, PriceSource,
            QuantityKg, ZoneSlug,
        },
    },
    shared::Phone,
};

pub const SELLER: &str = "+9647501234567";
pub const BUYER: &str = "+9647701112233";
pub const OTHER_BUYER: &str = "+9647709998877";

/// The id of the market every fixture listing is at.
pub const MARKET_ID: i32 = 1;
pub const MARKET: &str = "sulaymaniyah";

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RepositoryCall {
    IsCropTraded {
        crop: Crop,
    },
    FindMarkets,
    FindMarketBySlug {
        slug: String,
    },
    FindLatestPriceDay {
        market_id: i32,
    },
    FindPricesOn {
        market_id: i32,
        day: NaiveDate,
    },
    FindPricesBetween {
        market_ids: Vec<i32>,
        crops: Vec<Crop>,
        from: NaiveDate,
        to: NaiveDate,
    },
    UpsertPrice {
        market_id: i32,
        crop: Crop,
        day: NaiveDate,
    },
    FindListings {
        filter: ListingFilter,
        nearest_first: bool,
        page: u64,
        rows_per_page: u64,
    },
    FindListingById {
        id: i32,
    },
    FindListingsByIds {
        ids: Vec<i32>,
    },
    FindListingsBySeller {
        seller: String,
    },
    CountOpenListingsBySeller {
        seller: String,
    },
    CreateListing,
    CancelListing {
        id: i32,
    },
    SellListing {
        id: i32,
    },
    FindOffersByListings {
        listing_ids: Vec<i32>,
    },
    FindOffersByBuyer {
        buyer: String,
    },
    PlaceOffer {
        listing_id: i32,
        buyer: String,
    },
    AcceptOffer {
        listing_id: i32,
        offer_id: i32,
    },
    FindDeals {
        market_id: Option<i32>,
        day: NaiveDate,
    },
    CreateMarket {
        slug: String,
    },
    UpdateMarket {
        slug: String,
    },
    DeleteMarket {
        slug: String,
    },
    FindStoredPrices {
        filter: StoredPriceFilter,
        page: u64,
        rows_per_page: u64,
    },
    CreatePrice {
        market_id: i32,
        crop: Crop,
        day: NaiveDate,
    },
    UpdatePrice {
        market_id: i32,
        crop: Crop,
        day: NaiveDate,
    },
    DeletePrice {
        market: String,
        crop: Crop,
        day: NaiveDate,
    },
    FindListingsForModeration {
        filter: ModerationFilter,
        page: u64,
        rows_per_page: u64,
    },
    CloseListing {
        id: i32,
    },
    DeleteListing {
        id: i32,
    },
}

#[derive(Debug)]
struct Store {
    markets: Vec<Market>,
    /// What another request stores just before the next `close_listing` or
    /// `sell_listing`.
    rival: Option<Listing>,
    prices: Vec<Price>,
    listings: Vec<Listing>,
    offers: Vec<Offer>,
    /// Idempotency keys of posted listings, with the listing each created.
    listing_keys: Vec<(String, i32)>,
    fail_with_database_error: bool,
}

impl Default for Store {
    fn default() -> Self {
        Self {
            markets: markets(),
            rival: None,
            prices: Vec::new(),
            listings: Vec::new(),
            offers: Vec::new(),
            listing_keys: Vec::new(),
            fail_with_database_error: false,
        }
    }
}

/// Keeps what it is given in memory and applies writes the way the Postgres
/// repository does, so a test can read the state a use case left behind as
/// well as the calls it made.
#[derive(Debug, Clone, Default)]
pub struct FakeAlwaRepository {
    store: Arc<Mutex<Store>>,
    calls: Arc<Mutex<Vec<RepositoryCall>>>,
}

impl FakeAlwaRepository {
    /// Holds the two markets of `markets()` and nothing else.
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_listing(self, listing: Listing) -> Self {
        self.store
            .lock()
            .expect("store lock")
            .listings
            .push(listing);
        self
    }

    pub fn with_offer(self, offer: Offer) -> Self {
        self.store.lock().expect("store lock").offers.push(offer);
        self
    }

    pub fn with_price(self, price: Price) -> Self {
        self.store.lock().expect("store lock").prices.push(price);
        self
    }

    /// Another request gets to the listing first: `rival` is what it stores
    /// between this use case's read and its `close_listing` or
    /// `sell_listing`.
    pub fn with_rival_write(self, rival: Listing) -> Self {
        self.store.lock().expect("store lock").rival = Some(rival);
        self
    }

    pub fn failing() -> Self {
        let fake = Self::new();
        fake.store
            .lock()
            .expect("store lock")
            .fail_with_database_error = true;
        fake
    }

    pub fn calls(&self) -> Vec<RepositoryCall> {
        self.calls.lock().expect("calls lock").clone()
    }

    pub fn wrote(&self) -> bool {
        self.calls().iter().any(|call| {
            matches!(
                call,
                RepositoryCall::UpsertPrice { .. }
                    | RepositoryCall::CreateListing
                    | RepositoryCall::CancelListing { .. }
                    | RepositoryCall::SellListing { .. }
                    | RepositoryCall::PlaceOffer { .. }
                    | RepositoryCall::AcceptOffer { .. }
                    | RepositoryCall::CreateMarket { .. }
                    | RepositoryCall::UpdateMarket { .. }
                    | RepositoryCall::DeleteMarket { .. }
                    | RepositoryCall::CreatePrice { .. }
                    | RepositoryCall::UpdatePrice { .. }
                    | RepositoryCall::DeletePrice { .. }
                    | RepositoryCall::CloseListing { .. }
                    | RepositoryCall::DeleteListing { .. }
            )
        })
    }

    pub fn stored_listing(&self, id: i32) -> Option<Listing> {
        let store = self.store.lock().expect("store lock");

        store
            .listings
            .iter()
            .find(|listing| *listing.id() == Some(id))
            .cloned()
    }

    pub fn stored_markets(&self) -> Vec<Market> {
        self.store.lock().expect("store lock").markets.clone()
    }

    pub fn stored_offers(&self) -> Vec<Offer> {
        self.store.lock().expect("store lock").offers.clone()
    }

    pub fn stored_prices(&self) -> Vec<Price> {
        self.store.lock().expect("store lock").prices.clone()
    }

    fn record(&self, call: RepositoryCall) {
        self.calls.lock().expect("calls lock").push(call);
    }

    fn guard(&self) -> Result<(), AppError> {
        if self
            .store
            .lock()
            .expect("store lock")
            .fail_with_database_error
        {
            return Err(crate::app::AppError::DatabaseError("fake".to_string()).into());
        }

        Ok(())
    }
}

#[async_trait]
impl AlwaRepository for FakeAlwaRepository {
    async fn find_markets(&self) -> Result<Vec<Market>, AppError> {
        self.record(RepositoryCall::FindMarkets);
        self.guard()?;

        Ok(self.stored_markets())
    }

    async fn find_market_by_slug(&self, slug: &MarketSlug) -> Result<Option<Market>, AppError> {
        self.record(RepositoryCall::FindMarketBySlug {
            slug: String::from(slug),
        });
        self.guard()?;

        Ok(self
            .stored_markets()
            .into_iter()
            .find(|market| market.slug() == slug))
    }

    async fn find_latest_price_day(&self, market_id: i32) -> Result<Option<NaiveDate>, AppError> {
        self.record(RepositoryCall::FindLatestPriceDay { market_id });
        self.guard()?;

        let store = self.store.lock().expect("store lock");

        Ok(store
            .prices
            .iter()
            .filter(|price| *price.market_id() == market_id)
            .map(|price| *price.day())
            .max())
    }

    async fn find_prices_on(&self, market_id: i32, day: NaiveDate) -> Result<Vec<Price>, AppError> {
        self.record(RepositoryCall::FindPricesOn { market_id, day });
        self.guard()?;

        let store = self.store.lock().expect("store lock");

        Ok(store
            .prices
            .iter()
            .filter(|price| *price.market_id() == market_id && *price.day() == day)
            .cloned()
            .collect())
    }

    async fn find_prices_between(
        &self,
        market_ids: &[i32],
        crops: &[Crop],
        from: NaiveDate,
        to: NaiveDate,
    ) -> Result<Vec<Price>, AppError> {
        self.record(RepositoryCall::FindPricesBetween {
            market_ids: market_ids.to_vec(),
            crops: crops.to_vec(),
            from,
            to,
        });
        self.guard()?;

        let store = self.store.lock().expect("store lock");

        let mut prices: Vec<Price> = store
            .prices
            .iter()
            .filter(|price| market_ids.contains(price.market_id()))
            .filter(|price| crops.contains(price.crop()))
            .filter(|price| *price.day() >= from && *price.day() <= to)
            .cloned()
            .collect();
        prices.sort_by_key(|price| *price.day());

        Ok(prices)
    }

    async fn upsert_price(&self, price: &Price) -> Result<Price, AppError> {
        self.record(RepositoryCall::UpsertPrice {
            market_id: *price.market_id(),
            crop: *price.crop(),
            day: *price.day(),
        });
        self.guard()?;

        let stored = Price::rehydrate(
            1,
            *price.market_id(),
            *price.crop(),
            *price.day(),
            *price.price(),
            *price.fixed(),
            price.source().clone(),
            *price.updated_at(),
        );

        let mut store = self.store.lock().expect("store lock");
        store.prices.retain(|other| {
            (other.market_id(), other.crop(), other.day())
                != (price.market_id(), price.crop(), price.day())
        });
        store.prices.push(stored.clone());

        Ok(stored)
    }

    async fn find_listings(
        &self,
        filter: &ListingFilter,
        near: Option<&GeoPoint>,
        now: DateTime<Utc>,
        pagination: &Pagination,
    ) -> Result<(Vec<Listing>, u64), AppError> {
        self.record(RepositoryCall::FindListings {
            filter: *filter,
            nearest_first: near.is_some(),
            page: *pagination.page(),
            rows_per_page: *pagination.rows_per_page(),
        });
        self.guard()?;

        let store = self.store.lock().expect("store lock");

        let mut matching: Vec<Listing> = store
            .listings
            .iter()
            .filter(|listing| listing.status_at(now) == filter.status)
            .filter(|listing| {
                filter
                    .market_id
                    .is_none_or(|market_id| *listing.market_id() == Some(market_id))
            })
            .filter(|listing| filter.crop.is_none_or(|crop| *listing.crop() == crop))
            .cloned()
            .collect();
        let count = matching.len() as u64;

        // Nearest first, a listing without a place last, as the database
        // orders them.
        if let Some(from) = near {
            let km = |listing: &Listing| {
                listing
                    .point()
                    .map_or(f64::INFINITY, |point| from.km_to(&point))
            };

            matching.sort_by(|one, other| km(one).total_cmp(&km(other)));
        }

        Ok((
            matching
                .into_iter()
                .skip(pagination.skip() as usize)
                .take(*pagination.rows_per_page() as usize)
                .collect(),
            count,
        ))
    }

    async fn find_listing_by_id(&self, id: i32) -> Result<Option<Listing>, AppError> {
        self.record(RepositoryCall::FindListingById { id });
        self.guard()?;

        Ok(self.stored_listing(id))
    }

    async fn find_listings_by_ids(&self, ids: &[i32]) -> Result<Vec<Listing>, AppError> {
        self.record(RepositoryCall::FindListingsByIds { ids: ids.to_vec() });
        self.guard()?;

        let store = self.store.lock().expect("store lock");

        Ok(store
            .listings
            .iter()
            .filter(|listing| ids.contains(&listing.id().unwrap_or_default()))
            .cloned()
            .collect())
    }

    async fn find_listings_by_seller(&self, seller: &Phone) -> Result<Vec<Listing>, AppError> {
        self.record(RepositoryCall::FindListingsBySeller {
            seller: String::from(seller),
        });
        self.guard()?;

        let store = self.store.lock().expect("store lock");

        Ok(store
            .listings
            .iter()
            .filter(|listing| listing.is_sold_by(seller))
            .cloned()
            .collect())
    }

    async fn count_open_listings_by_seller(
        &self,
        seller: &Phone,
        now: DateTime<Utc>,
    ) -> Result<u64, AppError> {
        self.record(RepositoryCall::CountOpenListingsBySeller {
            seller: String::from(seller),
        });
        self.guard()?;

        let store = self.store.lock().expect("store lock");

        Ok(store
            .listings
            .iter()
            .filter(|listing| listing.is_sold_by(seller) && listing.is_open_at(now))
            .count() as u64)
    }

    async fn find_listing_by_idempotency_key(
        &self,
        _seller: &Phone,
        key: &IdempotencyKey,
    ) -> Result<Option<Listing>, AppError> {
        self.guard()?;

        let store = self.store.lock().expect("store lock");

        Ok(store
            .listing_keys
            .iter()
            .find(|(stored, _)| stored == key.as_str())
            .and_then(|(_, id)| store.listings.iter().find(|one| *one.id() == Some(*id)))
            .cloned())
    }

    async fn create_listing(
        &self,
        entity: &Listing,
        idempotency_key: Option<&IdempotencyKey>,
    ) -> Result<Listing, AppError> {
        self.record(RepositoryCall::CreateListing);
        self.guard()?;

        let mut store = self.store.lock().expect("store lock");
        let created = listing_with(entity, 1_000 + store.listings.len() as i32);
        store.listings.push(created.clone());

        if let (Some(key), Some(id)) = (idempotency_key, *created.id()) {
            store.listing_keys.push((key.as_str().to_string(), id));
        }

        Ok(created)
    }

    async fn cancel_listing(&self, entity: &Listing) -> Result<(), AppError> {
        let id = entity.id().unwrap_or_default();

        self.record(RepositoryCall::CancelListing { id });
        self.guard()?;

        let mut store = self.store.lock().expect("store lock");

        for stored in &mut store.listings {
            if *stored.id() == Some(id) {
                *stored = entity.clone();
            }
        }

        Ok(())
    }

    async fn sell_listing(&self, entity: &Listing) -> Result<(), AppError> {
        let id = entity.id().unwrap_or_default();

        self.record(RepositoryCall::SellListing { id });
        self.guard()?;

        let mut store = self.store.lock().expect("store lock");

        if let Some(rival) = store.rival.take() {
            for stored in &mut store.listings {
                if stored.id() == rival.id() {
                    *stored = rival.clone();
                }
            }
        }

        let still_open = store
            .listings
            .iter()
            .any(|stored| *stored.id() == Some(id) && *stored.status() == ListingStatus::Open);

        if !still_open {
            return Err(AlwaError::ListingNotOpen.into());
        }

        for stored in &mut store.listings {
            if *stored.id() == Some(id) {
                *stored = entity.clone();
            }
        }

        for stored in &mut store.offers {
            if *stored.listing_id() == id && stored.is_open() {
                *stored = offer_with(
                    stored,
                    stored.id().unwrap_or_default(),
                    OfferStatus::Declined,
                    *entity.updated_at(),
                );
            }
        }

        Ok(())
    }

    async fn find_offers_by_listings(&self, listing_ids: &[i32]) -> Result<Vec<Offer>, AppError> {
        self.record(RepositoryCall::FindOffersByListings {
            listing_ids: listing_ids.to_vec(),
        });
        self.guard()?;

        let store = self.store.lock().expect("store lock");

        Ok(store
            .offers
            .iter()
            .filter(|offer| listing_ids.contains(offer.listing_id()))
            .cloned()
            .collect())
    }

    async fn find_offers_by_buyer(&self, buyer: &Phone) -> Result<Vec<Offer>, AppError> {
        self.record(RepositoryCall::FindOffersByBuyer {
            buyer: String::from(buyer),
        });
        self.guard()?;

        let store = self.store.lock().expect("store lock");

        Ok(store
            .offers
            .iter()
            .filter(|offer| offer.is_by(buyer))
            .cloned()
            .collect())
    }

    async fn place_offer(&self, entity: &Offer) -> Result<Offer, AppError> {
        self.record(RepositoryCall::PlaceOffer {
            listing_id: *entity.listing_id(),
            buyer: String::from(entity.buyer_phone()),
        });
        self.guard()?;

        let mut store = self.store.lock().expect("store lock");

        for stored in &mut store.offers {
            if stored.listing_id() == entity.listing_id()
                && stored.is_by(entity.buyer_phone())
                && stored.is_open()
            {
                *stored = offer_with(
                    stored,
                    stored.id().unwrap_or_default(),
                    OfferStatus::Withdrawn,
                    *entity.created_at(),
                );
            }
        }

        let placed = offer_with(
            entity,
            1_000 + store.offers.len() as i32,
            *entity.status(),
            *entity.updated_at(),
        );
        store.offers.push(placed.clone());

        Ok(placed)
    }

    async fn accept_offer(&self, listing: &Listing, accepted: &Offer) -> Result<(), AppError> {
        let listing_id = listing.id().unwrap_or_default();

        self.record(RepositoryCall::AcceptOffer {
            listing_id,
            offer_id: accepted.id().unwrap_or_default(),
        });
        self.guard()?;

        let mut store = self.store.lock().expect("store lock");

        let still_open = store.listings.iter().any(|stored| {
            *stored.id() == Some(listing_id) && *stored.status() == ListingStatus::Open
        });

        if !still_open {
            return Err(AlwaError::ListingNotOpen.into());
        }

        for stored in &mut store.listings {
            if *stored.id() == Some(listing_id) {
                *stored = listing.clone();
            }
        }

        for stored in &mut store.offers {
            if stored.id() == accepted.id() {
                *stored = accepted.clone();
            } else if *stored.listing_id() == listing_id && stored.is_open() {
                *stored = offer_with(
                    stored,
                    stored.id().unwrap_or_default(),
                    OfferStatus::Declined,
                    *accepted.updated_at(),
                );
            }
        }

        Ok(())
    }

    async fn find_deals(
        &self,
        market_id: Option<i32>,
        day: NaiveDate,
    ) -> Result<Vec<Deal>, AppError> {
        self.record(RepositoryCall::FindDeals { market_id, day });
        self.guard()?;

        let store = self.store.lock().expect("store lock");

        Ok(store
            .offers
            .iter()
            .filter(|offer| offer.updated_at().date_naive() == day)
            .filter_map(|offer| {
                let listing = store
                    .listings
                    .iter()
                    .find(|listing| offer.is_on(listing))
                    .filter(|listing| {
                        market_id.is_none_or(|id| *listing.market_id() == Some(id))
                    })?;

                Deal::new(listing.clone(), offer.clone())
            })
            .collect())
    }

    async fn create_market(
        &self,
        slug: &MarketSlug,
        names: &MarketNames,
        point: Option<&GeoPoint>,
    ) -> Result<Option<Market>, AppError> {
        self.record(RepositoryCall::CreateMarket {
            slug: String::from(slug),
        });
        self.guard()?;

        let mut store = self.store.lock().expect("store lock");

        if store.markets.iter().any(|market| market.slug() == slug) {
            return Ok(None);
        }

        let created = Market::rehydrate(
            100 + store.markets.len() as i32,
            slug.clone(),
            String::from(&names.name_en),
            String::from(&names.name_ku),
        )
        .located(point.copied());
        store.markets.push(created.clone());

        Ok(Some(created))
    }

    async fn update_market(
        &self,
        slug: &MarketSlug,
        names: &MarketNames,
        point: Option<&GeoPoint>,
    ) -> Result<Option<Market>, AppError> {
        self.record(RepositoryCall::UpdateMarket {
            slug: String::from(slug),
        });
        self.guard()?;

        let mut store = self.store.lock().expect("store lock");

        let Some(stored) = store.markets.iter_mut().find(|one| one.slug() == slug) else {
            return Ok(None);
        };

        *stored = Market::rehydrate(
            *stored.id(),
            slug.clone(),
            String::from(&names.name_en),
            String::from(&names.name_ku),
        )
        .located(point.copied().or(*stored.point()));

        Ok(Some(stored.clone()))
    }

    async fn delete_market(&self, slug: &MarketSlug) -> Result<bool, AppError> {
        self.record(RepositoryCall::DeleteMarket {
            slug: String::from(slug),
        });
        self.guard()?;

        let mut store = self.store.lock().expect("store lock");

        let Some(id) = store
            .markets
            .iter()
            .find(|market| market.slug() == slug)
            .map(|market| *market.id())
        else {
            return Ok(false);
        };

        let in_use = store.prices.iter().any(|price| *price.market_id() == id)
            || store
                .listings
                .iter()
                .any(|listing| *listing.market_id() == Some(id));

        if in_use {
            return Err(AlwaError::MarketInUse.into());
        }

        store.markets.retain(|market| market.slug() != slug);

        Ok(true)
    }

    async fn find_stored_prices(
        &self,
        filter: &StoredPriceFilter,
        pagination: &Pagination,
    ) -> Result<(Vec<Price>, u64), AppError> {
        self.record(RepositoryCall::FindStoredPrices {
            filter: *filter,
            page: *pagination.page(),
            rows_per_page: *pagination.rows_per_page(),
        });
        self.guard()?;

        let store = self.store.lock().expect("store lock");

        let mut matching: Vec<Price> = store
            .prices
            .iter()
            .filter(|price| *price.market_id() == filter.market_id)
            .filter(|price| filter.crop.is_none_or(|crop| *price.crop() == crop))
            .filter(|price| filter.from.is_none_or(|from| *price.day() >= from))
            .filter(|price| filter.to.is_none_or(|to| *price.day() <= to))
            .cloned()
            .collect();
        matching.sort_by_key(|price| std::cmp::Reverse(*price.day()));
        let count = matching.len() as u64;

        Ok((
            matching
                .into_iter()
                .skip(pagination.skip() as usize)
                .take(*pagination.rows_per_page() as usize)
                .collect(),
            count,
        ))
    }

    async fn create_price(&self, price: &Price) -> Result<Option<Price>, AppError> {
        self.record(RepositoryCall::CreatePrice {
            market_id: *price.market_id(),
            crop: *price.crop(),
            day: *price.day(),
        });
        self.guard()?;

        let mut store = self.store.lock().expect("store lock");

        if store.prices.iter().any(|other| same_key(other, price)) {
            return Ok(None);
        }

        let created = price_with(price, 1_000 + store.prices.len() as i32);
        store.prices.push(created.clone());

        Ok(Some(created))
    }

    async fn update_price(&self, price: &Price) -> Result<Option<Price>, AppError> {
        self.record(RepositoryCall::UpdatePrice {
            market_id: *price.market_id(),
            crop: *price.crop(),
            day: *price.day(),
        });
        self.guard()?;

        let mut store = self.store.lock().expect("store lock");

        let Some(stored) = store.prices.iter_mut().find(|other| same_key(other, price)) else {
            return Ok(None);
        };

        *stored = price_with(price, stored.id().unwrap_or_default());

        Ok(Some(stored.clone()))
    }

    async fn delete_price(
        &self,
        market: &MarketSlug,
        crop: Crop,
        day: NaiveDate,
    ) -> Result<bool, AppError> {
        self.record(RepositoryCall::DeletePrice {
            market: String::from(market),
            crop,
            day,
        });
        self.guard()?;

        let mut store = self.store.lock().expect("store lock");

        let market_id = store
            .markets
            .iter()
            .find(|one| one.slug() == market)
            .map(|one| *one.id());
        let before = store.prices.len();

        store.prices.retain(|price| {
            Some(*price.market_id()) != market_id || *price.crop() != crop || *price.day() != day
        });

        Ok(store.prices.len() < before)
    }

    async fn find_listings_for_moderation(
        &self,
        filter: &ModerationFilter,
        now: DateTime<Utc>,
        pagination: &Pagination,
    ) -> Result<(Vec<Listing>, u64), AppError> {
        self.record(RepositoryCall::FindListingsForModeration {
            filter: filter.clone(),
            page: *pagination.page(),
            rows_per_page: *pagination.rows_per_page(),
        });
        self.guard()?;

        let store = self.store.lock().expect("store lock");

        let matching: Vec<Listing> = store
            .listings
            .iter()
            .filter(|listing| {
                filter
                    .status
                    .is_none_or(|status| listing.status_at(now) == status)
            })
            .filter(|listing| {
                filter
                    .market_id
                    .is_none_or(|market_id| *listing.market_id() == Some(market_id))
            })
            .filter(|listing| filter.crop.is_none_or(|crop| *listing.crop() == crop))
            .filter(|listing| {
                filter
                    .seller
                    .as_ref()
                    .is_none_or(|seller| listing.is_sold_by(seller))
            })
            .cloned()
            .collect();
        let count = matching.len() as u64;

        Ok((
            matching
                .into_iter()
                .skip(pagination.skip() as usize)
                .take(*pagination.rows_per_page() as usize)
                .collect(),
            count,
        ))
    }

    async fn close_listing(&self, entity: &Listing) -> Result<(), AppError> {
        let id = entity.id().unwrap_or_default();

        self.record(RepositoryCall::CloseListing { id });
        self.guard()?;

        let mut store = self.store.lock().expect("store lock");

        if let Some(rival) = store.rival.take() {
            for stored in &mut store.listings {
                if stored.id() == rival.id() {
                    *stored = rival.clone();
                }
            }
        }

        let still_open = store
            .listings
            .iter()
            .any(|stored| *stored.id() == Some(id) && *stored.status() == ListingStatus::Open);

        if !still_open {
            return Err(AlwaError::ListingNotOpen.into());
        }

        for stored in &mut store.listings {
            if *stored.id() == Some(id) {
                *stored = entity.clone();
            }
        }

        for stored in &mut store.offers {
            if *stored.listing_id() == id && stored.is_open() {
                *stored = offer_with(
                    stored,
                    stored.id().unwrap_or_default(),
                    OfferStatus::Declined,
                    *entity.updated_at(),
                );
            }
        }

        Ok(())
    }

    async fn delete_listing(&self, id: i32) -> Result<bool, AppError> {
        self.record(RepositoryCall::DeleteListing { id });
        self.guard()?;

        let mut store = self.store.lock().expect("store lock");

        if store
            .offers
            .iter()
            .any(|offer| *offer.listing_id() == id && offer.is_accepted())
        {
            return Err(AlwaError::ListingHasDeal.into());
        }

        let before = store.listings.len();
        store.listings.retain(|listing| *listing.id() != Some(id));
        store.offers.retain(|offer| *offer.listing_id() != id);

        Ok(store.listings.len() < before)
    }

    async fn is_crop_traded(&self, crop: Crop) -> Result<bool, AppError> {
        self.record(RepositoryCall::IsCropTraded { crop });
        self.guard()?;

        let store = self.store.lock().expect("store lock");

        Ok(store.listings.iter().any(|listing| *listing.crop() == crop)
            || store.prices.iter().any(|price| *price.crop() == crop))
    }
}

/// Stands in for the crops feature: the crops staff have switched on.
#[derive(Debug, Clone, Default)]
pub struct FakeCropDirectory {
    active: Vec<Crop>,
    failing: bool,
    asked: Arc<Mutex<u32>>,
}

impl FakeCropDirectory {
    /// The alwa crops that were fixed in code before staff kept the list.
    pub fn seeded() -> Self {
        Self::with(&[
            "wheat",
            "barley",
            "tomato",
            "cucumber",
            "potato",
            "onion",
            "watermelon",
            "grape",
            "olive",
            "sunflower",
            "chickpea",
            "pomegranate",
            "okra",
            "eggplant",
            "pepper",
            "apple",
        ])
    }

    pub fn with(codes: &[&str]) -> Self {
        Self {
            active: codes.iter().map(|code| Crop::of(code)).collect(),
            ..Self::default()
        }
    }

    pub fn failing() -> Self {
        Self {
            failing: true,
            ..Self::default()
        }
    }

    /// How many times the list was asked for.
    pub fn asked(&self) -> u32 {
        *self.asked.lock().expect("asked lock")
    }
}

#[async_trait]
impl CropDirectory for FakeCropDirectory {
    async fn active(&self) -> Result<ActiveCrops, AppError> {
        *self.asked.lock().expect("asked lock") += 1;

        if self.failing {
            return Err(crate::app::AppError::InternalServerError.into());
        }

        Ok(ActiveCrops::new(self.active.iter().copied()))
    }
}

/// Answers every point with one zone, or with none, and remembers the
/// points it was asked about.
#[derive(Debug, Clone, Default)]
pub struct FakeZoneLocator {
    zone: Option<String>,
    failing: bool,
    asked: Arc<Mutex<Vec<GeoPoint>>>,
}

impl FakeZoneLocator {
    pub fn everywhere(zone: &str) -> Self {
        Self {
            zone: Some(zone.to_string()),
            ..Self::default()
        }
    }

    pub fn nowhere() -> Self {
        Self::default()
    }

    pub fn failing() -> Self {
        Self {
            failing: true,
            ..Self::default()
        }
    }

    pub fn asked(&self) -> Vec<GeoPoint> {
        self.asked.lock().expect("asked lock").clone()
    }
}

#[async_trait]
impl ZoneLocator for FakeZoneLocator {
    async fn zone_of(&self, point: &GeoPoint) -> Result<Option<ZoneSlug>, AppError> {
        self.asked.lock().expect("asked lock").push(*point);

        if self.failing {
            return Err(crate::app::AppError::InternalServerError.into());
        }

        Ok(self
            .zone
            .clone()
            .map(|zone| ZoneSlug::new(zone).expect("zone slug")))
    }
}

fn same_key(one: &Price, other: &Price) -> bool {
    (one.market_id(), one.crop(), one.day()) == (other.market_id(), other.crop(), other.day())
}

fn price_with(entity: &Price, id: i32) -> Price {
    Price::rehydrate(
        id,
        *entity.market_id(),
        *entity.crop(),
        *entity.day(),
        *entity.price(),
        *entity.fixed(),
        entity.source().clone(),
        *entity.updated_at(),
    )
}

fn listing_with(entity: &Listing, id: i32) -> Listing {
    Listing::rehydrate(
        id,
        entity.seller_phone().clone(),
        entity.seller_name().clone(),
        *entity.crop(),
        *entity.quantity(),
        *entity.asking_price(),
        *entity.grade(),
        *entity.pickup(),
        *entity.market_id(),
        entity.market().clone(),
        entity.zone_slug().clone(),
        entity.note().clone(),
        *entity.closes_at(),
        *entity.status(),
        *entity.created_at(),
        *entity.updated_at(),
    )
    .placed_at(*entity.point())
}

fn offer_with(entity: &Offer, id: i32, status: OfferStatus, updated_at: DateTime<Utc>) -> Offer {
    Offer::rehydrate(
        id,
        *entity.listing_id(),
        entity.buyer_phone().clone(),
        entity.buyer_name().clone(),
        *entity.buyer_kind(),
        *entity.quantity(),
        *entity.price(),
        status,
        *entity.created_at(),
        updated_at,
    )
}

pub fn phone(value: &str) -> Phone {
    Phone::new(value.to_string()).expect("phone")
}

pub fn auth_context(phone_number: &str) -> AuthContext {
    AuthContext::new(User::new(phone(phone_number)), "token".to_string())
}

pub fn market_slug(value: &str) -> MarketSlug {
    MarketSlug::new(value.to_string()).expect("slug")
}

pub fn point(lat: f64, lon: f64) -> GeoPoint {
    GeoPoint::in_region(lat, lon).expect("point")
}

/// Where the two fixture markets are.
pub const SULAYMANIYAH: (f64, f64) = (35.5572, 45.4356);
pub const ERBIL: (f64, f64) = (36.1911, 44.0092);

pub fn markets() -> Vec<Market> {
    vec![
        Market::rehydrate(
            MARKET_ID,
            market_slug(MARKET),
            "Sulaymaniyah".to_string(),
            "سلێمانی".to_string(),
        )
        .located(Some(point(SULAYMANIYAH.0, SULAYMANIYAH.1))),
        Market::rehydrate(
            2,
            market_slug("erbil"),
            "Erbil".to_string(),
            "هەولێر".to_string(),
        )
        .located(Some(point(ERBIL.0, ERBIL.1))),
    ]
}

/// 500 kg of grade A tomato at 1,000 IQD, collected at the farm.
pub fn a_listing_draft(closes_at: DateTime<Utc>) -> ListingDraft {
    ListingDraft {
        seller_name: Some(DisplayName::new("Kak Azad".to_string()).expect("name")),
        crop: Crop::of("tomato"),
        quantity: QuantityKg::new(500).expect("quantity"),
        asking_price: PricePerKg::new(1_000).expect("price"),
        grade: Some(Grade::A),
        pickup: Some(Pickup::Farm),
        zone_slug: None,
        point: None,
        note: None,
        closes_at,
    }
}

/// A persisted listing of `SELLER` at the first market, made at `created_at`
/// and closing three days later.
pub fn a_listing(id: i32, created_at: DateTime<Utc>) -> Listing {
    let listing = Listing::new(
        phone(SELLER),
        Some(&markets()[0]),
        a_listing_draft(created_at + Duration::days(3)),
        created_at,
    )
    .expect("listing");

    listing_with(&listing, id)
}

/// A persisted listing of `SELLER` that is open now.
pub fn an_open_listing(id: i32) -> Listing {
    a_listing(id, Utc::now())
}

pub fn an_offer_draft(quantity: i64, price: i64) -> OfferDraft {
    OfferDraft {
        buyer_name: DisplayName::new("Bazaar shop".to_string()).expect("name"),
        buyer_kind: BuyerKind::Shop,
        quantity: QuantityKg::new(quantity).expect("quantity"),
        price: PricePerKg::new(price).expect("price"),
    }
}

/// A persisted open offer for the whole of `listing`.
pub fn an_open_offer(id: i32, listing: &Listing, buyer: &str, price: i64) -> Offer {
    let offer = listing
        .place_offer(
            phone(buyer),
            an_offer_draft(i64::from(listing.quantity().value()), price),
            &mut [],
            *listing.created_at(),
        )
        .expect("offer");

    offer_with(&offer, id, OfferStatus::Open, *offer.updated_at())
}

/// A listing `SELLER` sold to `BUYER` just now for 950 IQD, with the accepted
/// offer (id 1) and a declined one from `OTHER_BUYER` (id 2).
pub fn a_sold_listing(id: i32) -> (Listing, Vec<Offer>) {
    let mut listing = an_open_listing(id);
    let mut offers = vec![
        an_open_offer(1, &listing, BUYER, 950),
        an_open_offer(2, &listing, OTHER_BUYER, 900),
    ];

    listing
        .accept(&phone(SELLER), 1, &mut offers, Utc::now())
        .expect("accept");

    (listing, offers)
}

pub fn a_price(market_id: i32, crop: Crop, day: NaiveDate, value: i64, fixed: bool) -> Price {
    Price::rehydrate(
        1,
        market_id,
        crop,
        day,
        PricePerKg::new(value).expect("price"),
        fixed,
        PriceSource::new("alwa-board".to_string()).expect("source"),
        Utc::now(),
    )
}

pub fn market_names(name_en: &str, name_ku: &str) -> MarketNames {
    MarketNames {
        name_en: MarketName::new(name_en.to_string()).expect("name"),
        name_ku: MarketName::new(name_ku.to_string()).expect("name"),
    }
}
