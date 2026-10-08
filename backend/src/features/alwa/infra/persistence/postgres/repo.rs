use std::collections::HashMap;

use async_trait::async_trait;
use chrono::{DateTime, Duration, NaiveDate, NaiveTime, Utc};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, Condition, ConnectionTrait, DatabaseConnection, DbErr,
    EntityTrait, PaginatorTrait, QueryFilter, QueryOrder, QuerySelect, TransactionTrait,
    sea_query::{Expr, OnConflict},
};

use crate::{
    app::{AppError as GlobalAppError, Pagination},
    features::alwa::{
        app::{AlwaRepository, AppError, ListingFilter},
        domain::{
            AlwaError, Crop, Deal, Listing, ListingStatus, Market, MarketSlug, Offer, OfferStatus,
            Price,
        },
        infra::persistence::postgres::entities::{
            alwa_listings, alwa_markets, alwa_offers, alwa_prices,
        },
    },
    shared::Phone,
};

fn database_error(error: DbErr) -> AppError {
    tracing::error!(%error, "alwa repository query failed");

    GlobalAppError::DatabaseError(error.to_string()).into()
}

fn stored(status: ListingStatus) -> String {
    String::from(status)
}

/// The stored rows a reader sees under `status` at `now`. Nothing rewrites a
/// listing when its closing time passes, so "open" and "closed" both have to
/// look at the time as well as the column.
fn seen_as(status: ListingStatus, now: DateTime<Utc>) -> Condition {
    let now = now.naive_utc();
    let stored_open = alwa_listings::Column::Status.eq(stored(ListingStatus::Open));

    match status {
        ListingStatus::Open => Condition::all()
            .add(stored_open)
            .add(alwa_listings::Column::ClosesAt.gt(now)),
        ListingStatus::Closed => Condition::any()
            .add(alwa_listings::Column::Status.eq(stored(ListingStatus::Closed)))
            .add(
                Condition::all()
                    .add(stored_open)
                    .add(alwa_listings::Column::ClosesAt.lte(now)),
            ),
        ListingStatus::Sold | ListingStatus::Cancelled => {
            Condition::all().add(alwa_listings::Column::Status.eq(stored(status)))
        }
    }
}

#[derive(Debug)]
pub struct AlwaPostgresRepository {
    conn: DatabaseConnection,
}

impl AlwaPostgresRepository {
    pub fn new(conn: DatabaseConnection) -> Self {
        Self { conn }
    }

    /// A listing answer names its market by slug. There are only a handful
    /// of markets, so they are read whole instead of joined to every query.
    async fn market_slugs<C: ConnectionTrait>(
        conn: &C,
    ) -> Result<HashMap<i32, MarketSlug>, AppError> {
        alwa_markets::Entity::find()
            .all(conn)
            .await
            .map_err(database_error)?
            .into_iter()
            .map(|model| Ok((model.id, MarketSlug::new(model.slug)?)))
            .collect()
    }

    async fn listings_from<C: ConnectionTrait>(
        conn: &C,
        models: Vec<alwa_listings::Model>,
    ) -> Result<Vec<Listing>, AppError> {
        if models.is_empty() {
            return Ok(Vec::new());
        }

        let slugs = Self::market_slugs(conn).await?;

        models
            .into_iter()
            .map(|model| {
                let market = slugs.get(&model.market_id).cloned().ok_or_else(|| {
                    GlobalAppError::MissingValue(format!(
                        "Listing {} points at a market that is gone",
                        model.id
                    ))
                })?;

                Listing::try_from((model, market))
            })
            .collect()
    }

    /// Reads the listing row and holds it until the transaction ends, so two
    /// writers on the same listing take turns. Fails with `ListingNotOpen`
    /// when the row is no longer stored as open.
    async fn lock_open_listing<C: ConnectionTrait>(conn: &C, id: i32) -> Result<(), AppError> {
        let locked = alwa_listings::Entity::find_by_id(id)
            .lock_exclusive()
            .one(conn)
            .await
            .map_err(database_error)?;

        match locked {
            Some(model) if model.status == stored(ListingStatus::Open) => Ok(()),
            Some(_) => Err(AlwaError::ListingNotOpen.into()),
            None => Err(GlobalAppError::NotFound.into()),
        }
    }
}

fn persisted_id(id: Option<i32>, what: &str) -> Result<i32, AppError> {
    id.ok_or_else(|| {
        GlobalAppError::MissingValue(format!("Cannot update {what} that has not been persisted"))
            .into()
    })
}

#[async_trait]
impl AlwaRepository for AlwaPostgresRepository {
    async fn find_markets(&self) -> Result<Vec<Market>, AppError> {
        alwa_markets::Entity::find()
            .order_by_asc(alwa_markets::Column::Id)
            .all(&self.conn)
            .await
            .map_err(database_error)?
            .into_iter()
            .map(Market::try_from)
            .collect()
    }

    async fn find_market_by_slug(&self, slug: &MarketSlug) -> Result<Option<Market>, AppError> {
        let model = alwa_markets::Entity::find()
            .filter(alwa_markets::Column::Slug.eq(slug.as_str()))
            .one(&self.conn)
            .await
            .map_err(database_error)?;

        model.map(Market::try_from).transpose()
    }

    async fn find_latest_price_day(&self, market_id: i32) -> Result<Option<NaiveDate>, AppError> {
        let latest = alwa_prices::Entity::find()
            .filter(alwa_prices::Column::MarketId.eq(market_id))
            .order_by_desc(alwa_prices::Column::Day)
            .one(&self.conn)
            .await
            .map_err(database_error)?;

        Ok(latest.map(|model| model.day))
    }

    async fn find_prices_on(&self, market_id: i32, day: NaiveDate) -> Result<Vec<Price>, AppError> {
        alwa_prices::Entity::find()
            .filter(alwa_prices::Column::MarketId.eq(market_id))
            .filter(alwa_prices::Column::Day.eq(day))
            .order_by_asc(alwa_prices::Column::Crop)
            .all(&self.conn)
            .await
            .map_err(database_error)?
            .into_iter()
            .map(Price::try_from)
            .collect()
    }

    async fn find_prices_between(
        &self,
        market_ids: &[i32],
        crops: &[Crop],
        from: NaiveDate,
        to: NaiveDate,
    ) -> Result<Vec<Price>, AppError> {
        let crops: Vec<String> = crops.iter().map(|crop| String::from(*crop)).collect();

        alwa_prices::Entity::find()
            .filter(alwa_prices::Column::MarketId.is_in(market_ids.to_vec()))
            .filter(alwa_prices::Column::Crop.is_in(crops))
            .filter(alwa_prices::Column::Day.gte(from))
            .filter(alwa_prices::Column::Day.lte(to))
            .order_by_asc(alwa_prices::Column::Day)
            .order_by_asc(alwa_prices::Column::Id)
            .all(&self.conn)
            .await
            .map_err(database_error)?
            .into_iter()
            .map(Price::try_from)
            .collect()
    }

    async fn upsert_price(&self, price: &Price) -> Result<Price, AppError> {
        let crop = String::from(*price.crop());

        // Market, crop and day are the key, so the same day sent again
        // replaces the price instead of adding a second one.
        alwa_prices::Entity::insert(alwa_prices::ActiveModel::from(price))
            .on_conflict(
                OnConflict::columns([
                    alwa_prices::Column::MarketId,
                    alwa_prices::Column::Crop,
                    alwa_prices::Column::Day,
                ])
                .update_columns([
                    alwa_prices::Column::PriceIqdPerKg,
                    alwa_prices::Column::Fixed,
                    alwa_prices::Column::Source,
                    alwa_prices::Column::UpdatedAt,
                ])
                .to_owned(),
            )
            .exec(&self.conn)
            .await
            .map_err(database_error)?;

        let model = alwa_prices::Entity::find()
            .filter(alwa_prices::Column::MarketId.eq(*price.market_id()))
            .filter(alwa_prices::Column::Crop.eq(crop))
            .filter(alwa_prices::Column::Day.eq(*price.day()))
            .one(&self.conn)
            .await
            .map_err(database_error)?
            .ok_or_else(|| {
                GlobalAppError::MissingValue("The price just stored is gone".to_string())
            })?;

        Price::try_from(model)
    }

    async fn find_listings(
        &self,
        filter: &ListingFilter,
        now: DateTime<Utc>,
        pagination: &Pagination,
    ) -> Result<(Vec<Listing>, u64), AppError> {
        let mut query = alwa_listings::Entity::find().filter(seen_as(filter.status, now));

        if let Some(market_id) = filter.market_id {
            query = query.filter(alwa_listings::Column::MarketId.eq(market_id));
        }

        if let Some(crop) = filter.crop {
            query = query.filter(alwa_listings::Column::Crop.eq(String::from(crop)));
        }

        let count = query
            .clone()
            .count(&self.conn)
            .await
            .map_err(database_error)?;

        let models = query
            .order_by_desc(alwa_listings::Column::CreatedAt)
            .order_by_desc(alwa_listings::Column::Id)
            .offset(pagination.skip())
            .limit(*pagination.rows_per_page())
            .all(&self.conn)
            .await
            .map_err(database_error)?;

        Ok((Self::listings_from(&self.conn, models).await?, count))
    }

    async fn find_listing_by_id(&self, id: i32) -> Result<Option<Listing>, AppError> {
        let model = alwa_listings::Entity::find_by_id(id)
            .one(&self.conn)
            .await
            .map_err(database_error)?;

        Ok(Self::listings_from(&self.conn, model.into_iter().collect())
            .await?
            .into_iter()
            .next())
    }

    async fn find_listings_by_ids(&self, ids: &[i32]) -> Result<Vec<Listing>, AppError> {
        let models = alwa_listings::Entity::find()
            .filter(alwa_listings::Column::Id.is_in(ids.to_vec()))
            .all(&self.conn)
            .await
            .map_err(database_error)?;

        Self::listings_from(&self.conn, models).await
    }

    async fn find_listings_by_seller(&self, seller: &Phone) -> Result<Vec<Listing>, AppError> {
        let models = alwa_listings::Entity::find()
            .filter(alwa_listings::Column::SellerPhone.eq(seller.as_str()))
            .order_by_desc(alwa_listings::Column::CreatedAt)
            .order_by_desc(alwa_listings::Column::Id)
            .all(&self.conn)
            .await
            .map_err(database_error)?;

        Self::listings_from(&self.conn, models).await
    }

    async fn count_open_listings_by_seller(
        &self,
        seller: &Phone,
        now: DateTime<Utc>,
    ) -> Result<u64, AppError> {
        alwa_listings::Entity::find()
            .filter(alwa_listings::Column::SellerPhone.eq(seller.as_str()))
            .filter(seen_as(ListingStatus::Open, now))
            .count(&self.conn)
            .await
            .map_err(database_error)
    }

    async fn create_listing(&self, entity: &Listing) -> Result<Listing, AppError> {
        let model = alwa_listings::ActiveModel::from(entity)
            .insert(&self.conn)
            .await
            .map_err(database_error)?;

        Listing::try_from((model, entity.market().clone()))
    }

    async fn cancel_listing(&self, entity: &Listing) -> Result<(), AppError> {
        let id = persisted_id(*entity.id(), "a listing")?;

        // Only a row still stored as open is cancelled, so a deal made a
        // moment earlier is never overwritten.
        let result = alwa_listings::Entity::update_many()
            .col_expr(
                alwa_listings::Column::Status,
                Expr::value(stored(*entity.status())),
            )
            .col_expr(
                alwa_listings::Column::UpdatedAt,
                Expr::value(entity.updated_at().naive_utc()),
            )
            .filter(alwa_listings::Column::Id.eq(id))
            .filter(alwa_listings::Column::SellerPhone.eq(entity.seller_phone().as_str()))
            .filter(alwa_listings::Column::Status.eq(stored(ListingStatus::Open)))
            .exec(&self.conn)
            .await
            .map_err(database_error)?;

        if result.rows_affected == 0 {
            return Err(AlwaError::ListingNotOpen.into());
        }

        Ok(())
    }

    async fn find_offers_by_listings(&self, listing_ids: &[i32]) -> Result<Vec<Offer>, AppError> {
        alwa_offers::Entity::find()
            .filter(alwa_offers::Column::ListingId.is_in(listing_ids.to_vec()))
            .order_by_asc(alwa_offers::Column::Id)
            .all(&self.conn)
            .await
            .map_err(database_error)?
            .into_iter()
            .map(Offer::try_from)
            .collect()
    }

    async fn find_offers_by_buyer(&self, buyer: &Phone) -> Result<Vec<Offer>, AppError> {
        alwa_offers::Entity::find()
            .filter(alwa_offers::Column::BuyerPhone.eq(buyer.as_str()))
            .order_by_desc(alwa_offers::Column::CreatedAt)
            .order_by_desc(alwa_offers::Column::Id)
            .all(&self.conn)
            .await
            .map_err(database_error)?
            .into_iter()
            .map(Offer::try_from)
            .collect()
    }

    async fn place_offer(&self, entity: &Offer) -> Result<Offer, AppError> {
        let transaction = self.conn.begin().await.map_err(database_error)?;

        Self::lock_open_listing(&transaction, *entity.listing_id()).await?;

        // With the listing held, the buyer's earlier open offer is withdrawn
        // here rather than by id, so two offers sent at once cannot both
        // stay open.
        alwa_offers::Entity::update_many()
            .col_expr(
                alwa_offers::Column::Status,
                Expr::value(String::from(OfferStatus::Withdrawn)),
            )
            .col_expr(
                alwa_offers::Column::UpdatedAt,
                Expr::value(entity.created_at().naive_utc()),
            )
            .filter(alwa_offers::Column::ListingId.eq(*entity.listing_id()))
            .filter(alwa_offers::Column::BuyerPhone.eq(entity.buyer_phone().as_str()))
            .filter(alwa_offers::Column::Status.eq(String::from(OfferStatus::Open)))
            .exec(&transaction)
            .await
            .map_err(database_error)?;

        let model = alwa_offers::ActiveModel::from(entity)
            .insert(&transaction)
            .await
            .map_err(database_error)?;

        transaction.commit().await.map_err(database_error)?;

        Offer::try_from(model)
    }

    async fn accept_offer(&self, listing: &Listing, accepted: &Offer) -> Result<(), AppError> {
        let listing_id = persisted_id(*listing.id(), "a listing")?;
        let offer_id = persisted_id(*accepted.id(), "an offer")?;
        let at = accepted.updated_at().naive_utc();

        // Leaving early drops the transaction, which rolls it back.
        let transaction = self.conn.begin().await.map_err(database_error)?;

        Self::lock_open_listing(&transaction, listing_id).await?;

        alwa_listings::Entity::update_many()
            .col_expr(
                alwa_listings::Column::Status,
                Expr::value(stored(ListingStatus::Sold)),
            )
            .col_expr(
                alwa_listings::Column::UpdatedAt,
                Expr::value(listing.updated_at().naive_utc()),
            )
            .filter(alwa_listings::Column::Id.eq(listing_id))
            .exec(&transaction)
            .await
            .map_err(database_error)?;

        let taken = alwa_offers::Entity::update_many()
            .col_expr(
                alwa_offers::Column::Status,
                Expr::value(String::from(OfferStatus::Accepted)),
            )
            .col_expr(alwa_offers::Column::UpdatedAt, Expr::value(at))
            .filter(alwa_offers::Column::Id.eq(offer_id))
            .filter(alwa_offers::Column::ListingId.eq(listing_id))
            .filter(alwa_offers::Column::Status.eq(String::from(OfferStatus::Open)))
            .exec(&transaction)
            .await
            .map_err(database_error)?;

        if taken.rows_affected == 0 {
            return Err(AlwaError::OfferNotOpen.into());
        }

        alwa_offers::Entity::update_many()
            .col_expr(
                alwa_offers::Column::Status,
                Expr::value(String::from(OfferStatus::Declined)),
            )
            .col_expr(alwa_offers::Column::UpdatedAt, Expr::value(at))
            .filter(alwa_offers::Column::ListingId.eq(listing_id))
            .filter(alwa_offers::Column::Status.eq(String::from(OfferStatus::Open)))
            .exec(&transaction)
            .await
            .map_err(database_error)?;

        transaction.commit().await.map_err(database_error)?;

        Ok(())
    }

    async fn find_deals(
        &self,
        market_id: Option<i32>,
        day: NaiveDate,
    ) -> Result<Vec<Deal>, AppError> {
        let from = day.and_time(NaiveTime::MIN);
        let until = from + Duration::days(1);

        let mut query = alwa_offers::Entity::find()
            .find_also_related(alwa_listings::Entity)
            .filter(alwa_offers::Column::Status.eq(String::from(OfferStatus::Accepted)))
            .filter(alwa_offers::Column::UpdatedAt.gte(from))
            .filter(alwa_offers::Column::UpdatedAt.lt(until));

        if let Some(market_id) = market_id {
            query = query.filter(alwa_listings::Column::MarketId.eq(market_id));
        }

        let rows = query
            .order_by_desc(alwa_offers::Column::UpdatedAt)
            .order_by_desc(alwa_offers::Column::Id)
            .all(&self.conn)
            .await
            .map_err(database_error)?;

        let mut offers = Vec::with_capacity(rows.len());
        let mut listings = Vec::with_capacity(rows.len());

        for (offer, listing) in rows {
            if let Some(listing) = listing {
                offers.push(Offer::try_from(offer)?);
                listings.push(listing);
            }
        }

        let listings = Self::listings_from(&self.conn, listings).await?;

        Ok(listings
            .into_iter()
            .zip(offers)
            .filter_map(|(listing, offer)| Deal::new(listing, offer))
            .collect())
    }
}
