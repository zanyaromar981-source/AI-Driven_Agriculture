use async_trait::async_trait;
use chrono::{DateTime, NaiveDate, Utc};

use crate::{
    app::Pagination,
    features::alwa::{
        app::AppError,
        domain::{
            Crop, Deal, GeoPoint, IdempotencyKey, Listing, ListingStatus, Market, MarketNames,
            MarketSlug, Offer, Price,
        },
    },
    shared::Phone,
};

/// Which listings the public board shows.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ListingFilter {
    pub market_id: Option<i32>,
    pub crop: Option<Crop>,
    /// The status a reader sees, see `Listing::status_at`.
    pub status: ListingStatus,
}

/// Which stored prices of one market the dashboard lists.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct StoredPriceFilter {
    pub market_id: i32,
    pub crop: Option<Crop>,
    /// The first day, included.
    pub from: Option<NaiveDate>,
    /// The last day, included.
    pub to: Option<NaiveDate>,
}

/// Which listings the dashboard lists. Nothing set means every listing.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ModerationFilter {
    pub market_id: Option<i32>,
    pub crop: Option<Crop>,
    /// The status a reader sees, see `Listing::status_at`.
    pub status: Option<ListingStatus>,
    pub seller: Option<Phone>,
}

#[async_trait]
pub trait AlwaRepository: Send + Sync + std::fmt::Debug {
    /// Returns every alwa, in the order they were seeded.
    async fn find_markets(&self) -> Result<Vec<Market>, AppError>;

    async fn find_market_by_slug(&self, slug: &MarketSlug) -> Result<Option<Market>, AppError>;

    /// The latest day the market has any price for.
    async fn find_latest_price_day(&self, market_id: i32) -> Result<Option<NaiveDate>, AppError>;

    /// Returns the market's prices on one day, ordered by crop.
    async fn find_prices_on(&self, market_id: i32, day: NaiveDate) -> Result<Vec<Price>, AppError>;

    /// Returns the prices of the given crops at the given markets from
    /// `from` to `to`, both included, oldest first.
    async fn find_prices_between(
        &self,
        market_ids: &[i32],
        crops: &[Crop],
        from: NaiveDate,
        to: NaiveDate,
    ) -> Result<Vec<Price>, AppError>;

    /// Stores the price, replacing the one the market already has for that
    /// crop and day.
    async fn upsert_price(&self, price: &Price) -> Result<Price, AppError>;

    /// Returns one page of the board and how many listings match in all.
    /// With `near` the page is ordered by distance from that point, nearest
    /// first, and listings without a place come last; without it, and
    /// between listings equally far, newest first.
    async fn find_listings(
        &self,
        filter: &ListingFilter,
        near: Option<&GeoPoint>,
        now: DateTime<Utc>,
        pagination: &Pagination,
    ) -> Result<(Vec<Listing>, u64), AppError>;

    async fn find_listing_by_id(&self, id: i32) -> Result<Option<Listing>, AppError>;

    async fn find_listings_by_ids(&self, ids: &[i32]) -> Result<Vec<Listing>, AppError>;

    /// Returns the seller's listings of every status, newest first.
    async fn find_listings_by_seller(&self, seller: &Phone) -> Result<Vec<Listing>, AppError>;

    /// How many listings of the seller a reader sees as open at `now`.
    async fn count_open_listings_by_seller(
        &self,
        seller: &Phone,
        now: DateTime<Utc>,
    ) -> Result<u64, AppError>;

    /// Creates a new entity. `entity.id()` must be `None`; the database
    /// assigns the id.
    async fn create_listing(
        &self,
        entity: &Listing,
        idempotency_key: Option<&IdempotencyKey>,
    ) -> Result<Listing, AppError>;

    /// Returns the listing an earlier post with the same key created.
    async fn find_listing_by_idempotency_key(
        &self,
        seller: &Phone,
        key: &IdempotencyKey,
    ) -> Result<Option<Listing>, AppError>;

    /// Stores a cancelled listing. Fails with `ListingNotOpen` when the
    /// stored listing stopped being open in the meantime.
    async fn cancel_listing(&self, entity: &Listing) -> Result<(), AppError>;

    /// Stores a listing its seller marked sold and, in the same
    /// transaction, declines every open offer on it. Fails with
    /// `ListingNotOpen` when the stored listing stopped being open in the
    /// meantime, and then changes nothing.
    async fn sell_listing(&self, entity: &Listing) -> Result<(), AppError>;

    /// Returns every offer on the given listings.
    async fn find_offers_by_listings(&self, listing_ids: &[i32]) -> Result<Vec<Offer>, AppError>;

    /// Returns the buyer's offers of every status, newest first.
    async fn find_offers_by_buyer(&self, buyer: &Phone) -> Result<Vec<Offer>, AppError>;

    /// Creates a new offer and, in the same transaction, withdraws any open
    /// offer the same buyer has on the listing. Fails with `ListingNotOpen`
    /// when the stored listing stopped being open in the meantime.
    async fn place_offer(&self, entity: &Offer) -> Result<Offer, AppError>;

    /// Stores a deal in one transaction: the listing becomes sold, the
    /// accepted offer accepted, every other open offer on it declined. Fails
    /// with `ListingNotOpen` or `OfferNotOpen` when either stopped being open
    /// in the meantime, and then changes nothing.
    async fn accept_offer(&self, listing: &Listing, accepted: &Offer) -> Result<(), AppError>;

    /// Returns the deals made on one UTC day, latest first, at one market
    /// or at all of them.
    async fn find_deals(
        &self,
        market_id: Option<i32>,
        day: NaiveDate,
    ) -> Result<Vec<Deal>, AppError>;

    /// Creates the market unless one already has the slug: `None` then, and
    /// nothing is written.
    async fn create_market(
        &self,
        slug: &MarketSlug,
        names: &MarketNames,
        point: Option<&GeoPoint>,
    ) -> Result<Option<Market>, AppError>;

    /// Replaces the names of the market with this slug and, when `point` is
    /// given, its place; without one the stored place is kept. `None` when
    /// there is no such market.
    async fn update_market(
        &self,
        slug: &MarketSlug,
        names: &MarketNames,
        point: Option<&GeoPoint>,
    ) -> Result<Option<Market>, AppError>;

    /// Removes the market and says whether there was one. Fails with
    /// `MarketInUse` while a price or a listing still points at it.
    async fn delete_market(&self, slug: &MarketSlug) -> Result<bool, AppError>;

    /// Returns one page of a market's stored prices, newest day first, and
    /// how many match in all.
    async fn find_stored_prices(
        &self,
        filter: &StoredPriceFilter,
        pagination: &Pagination,
    ) -> Result<(Vec<Price>, u64), AppError>;

    /// Stores the price unless the market already has one for that crop and
    /// day: `None` then, and nothing is written.
    async fn create_price(&self, price: &Price) -> Result<Option<Price>, AppError>;

    /// Replaces the price the market has for that crop and day. `None` when
    /// it has none.
    async fn update_price(&self, price: &Price) -> Result<Option<Price>, AppError>;

    /// Removes one price and says whether there was one.
    async fn delete_price(
        &self,
        market: &MarketSlug,
        crop: Crop,
        day: NaiveDate,
    ) -> Result<bool, AppError>;

    /// Returns one page of the listings of every seller, newest first, and
    /// how many match in all.
    async fn find_listings_for_moderation(
        &self,
        filter: &ModerationFilter,
        now: DateTime<Utc>,
        pagination: &Pagination,
    ) -> Result<(Vec<Listing>, u64), AppError>;

    /// Stores a listing staff closed and, in the same transaction, declines
    /// every open offer on it. Fails with `ListingNotOpen` when the stored
    /// listing stopped being open in the meantime, and then changes nothing.
    async fn close_listing(&self, entity: &Listing) -> Result<(), AppError>;

    /// Removes the listing with its offers and says whether there was one.
    /// Fails with `ListingHasDeal` when an offer on it was accepted.
    async fn delete_listing(&self, id: i32) -> Result<bool, AppError>;

    /// Whether any listing, of any status, or any price names the crop. For
    /// the feature that keeps the crop list and may not remove a crop in
    /// use.
    async fn is_crop_traded(&self, crop: Crop) -> Result<bool, AppError>;
}
