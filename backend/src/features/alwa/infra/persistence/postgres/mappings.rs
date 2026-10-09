use sea_orm::ActiveValue::{NotSet, Set};

use crate::{
    features::alwa::{
        app::AppError,
        domain::{
            BuyerKind, Crop, DisplayName, GeoPoint, Grade, Listing, ListingStatus, Market,
            MarketSlug, Moderation, Note, Offer, OfferStatus, Pickup, Price, PricePerKg,
            PriceSource, ProductGroup, QuantityKg, Unit, ZoneSlug,
        },
        infra::persistence::postgres::entities::{
            alwa_listings, alwa_markets, alwa_offers, alwa_prices,
        },
    },
    shared::Phone,
};

impl TryFrom<alwa_markets::Model> for Market {
    type Error = AppError;

    fn try_from(model: alwa_markets::Model) -> Result<Self, Self::Error> {
        let point = GeoPoint::from_pair(model.lat, model.lon, GeoPoint::in_region)?;

        Ok(Market::rehydrate(
            model.id,
            MarketSlug::new(model.slug)?,
            model.name_en,
            model.name_ku,
        )
        .located(point))
    }
}

impl TryFrom<alwa_prices::Model> for Price {
    type Error = AppError;

    fn try_from(model: alwa_prices::Model) -> Result<Self, Self::Error> {
        Ok(Price::rehydrate(
            model.id,
            model.market_id,
            Crop::new(model.crop.as_str())?,
            model.day,
            PricePerKg::new(i64::from(model.price_iqd_per_kg))?,
            model.fixed,
            PriceSource::new(model.source)?,
            model.updated_at.and_utc(),
        )
        .per(Unit::try_from(model.unit.as_str())?))
    }
}

impl From<&Price> for alwa_prices::ActiveModel {
    fn from(price: &Price) -> Self {
        alwa_prices::ActiveModel {
            id: match *price.id() {
                Some(id) => Set(id),
                None => NotSet,
            },
            market_id: Set(*price.market_id()),
            crop: Set((*price.crop()).into()),
            day: Set(*price.day()),
            price_iqd_per_kg: Set(price.price().value()),
            fixed: Set(*price.fixed()),
            source: Set(price.source().into()),
            updated_at: Set(price.updated_at().naive_utc()),
            unit: Set((*price.unit()).into()),
        }
    }
}

/// The listing row and the slug of the market it points at, if any.
impl TryFrom<(alwa_listings::Model, Option<MarketSlug>)> for Listing {
    type Error = AppError;

    fn try_from(
        (model, market): (alwa_listings::Model, Option<MarketSlug>),
    ) -> Result<Self, Self::Error> {
        let point = GeoPoint::from_pair(model.lat, model.lon, GeoPoint::in_region)?;

        let moderation = match model.closed_by_staff_id {
            Some(staff_id) => Some(Moderation::new(
                staff_id,
                model.moderation_note.map(Note::new).transpose()?,
            )),
            None => None,
        };

        let listing = Listing::rehydrate(
            model.id,
            Phone::new(model.seller_phone)?,
            model.seller_name.map(DisplayName::new).transpose()?,
            Crop::new(model.crop.as_str())?,
            QuantityKg::new(i64::from(model.quantity_kg))?,
            PricePerKg::new(i64::from(model.asking_price_iqd_per_kg))?,
            model.grade.as_deref().map(Grade::try_from).transpose()?,
            model.pickup.as_deref().map(Pickup::try_from).transpose()?,
            model.market_id,
            market,
            model.zone_slug.map(ZoneSlug::new).transpose()?,
            model.note.map(Note::new).transpose()?,
            model.closes_at.and_utc(),
            ListingStatus::try_from(model.status.as_str())?,
            model.created_at.and_utc(),
            model.updated_at.and_utc(),
        )
        .placed_at(point)
        .sold_as(
            ProductGroup::try_from(model.grp.as_str())?,
            Unit::try_from(model.unit.as_str())?,
        );

        Ok(match moderation {
            Some(moderation) => listing.moderated(moderation),
            None => listing,
        })
    }
}

impl From<&Listing> for alwa_listings::ActiveModel {
    fn from(listing: &Listing) -> Self {
        alwa_listings::ActiveModel {
            id: match *listing.id() {
                Some(id) => Set(id),
                None => NotSet,
            },
            seller_phone: Set(listing.seller_phone().into()),
            seller_name: Set(listing.seller_name().as_ref().map(Into::into)),
            crop: Set((*listing.crop()).into()),
            quantity_kg: Set(listing.quantity().value()),
            asking_price_iqd_per_kg: Set(listing.asking_price().value()),
            grade: Set(listing.grade().map(Into::into)),
            pickup: Set(listing.pickup().map(Into::into)),
            market_id: Set(*listing.market_id()),
            zone_slug: Set(listing.zone_slug().as_ref().map(Into::into)),
            note: Set(listing.note().as_ref().map(Into::into)),
            closes_at: Set(listing.closes_at().naive_utc()),
            status: Set((*listing.status()).into()),
            created_at: Set(listing.created_at().naive_utc()),
            updated_at: Set(listing.updated_at().naive_utc()),
            // Set by the repository when a listing is created with a key.
            idempotency_key: NotSet,
            closed_by_staff_id: Set(listing
                .moderation()
                .as_ref()
                .map(|moderation| *moderation.staff_id())),
            moderation_note: Set(listing
                .moderation()
                .as_ref()
                .and_then(|moderation| moderation.note().as_ref().map(Into::into))),
            lat: Set(listing.point().map(|point| point.lat())),
            lon: Set(listing.point().map(|point| point.lon())),
            grp: Set((*listing.group()).into()),
            unit: Set((*listing.unit()).into()),
        }
    }
}

impl TryFrom<alwa_offers::Model> for Offer {
    type Error = AppError;

    fn try_from(model: alwa_offers::Model) -> Result<Self, Self::Error> {
        Ok(Offer::rehydrate(
            model.id,
            model.listing_id,
            Phone::new(model.buyer_phone)?,
            DisplayName::new(model.buyer_name)?,
            BuyerKind::try_from(model.buyer_kind.as_str())?,
            QuantityKg::new(i64::from(model.quantity_kg))?,
            PricePerKg::new(i64::from(model.price_iqd_per_kg))?,
            OfferStatus::try_from(model.status.as_str())?,
            model.created_at.and_utc(),
            model.updated_at.and_utc(),
        ))
    }
}

impl From<&Offer> for alwa_offers::ActiveModel {
    fn from(offer: &Offer) -> Self {
        alwa_offers::ActiveModel {
            id: match *offer.id() {
                Some(id) => Set(id),
                None => NotSet,
            },
            listing_id: Set(*offer.listing_id()),
            buyer_phone: Set(offer.buyer_phone().into()),
            buyer_name: Set(offer.buyer_name().into()),
            buyer_kind: Set((*offer.buyer_kind()).into()),
            quantity_kg: Set(offer.quantity().value()),
            price_iqd_per_kg: Set(offer.price().value()),
            status: Set((*offer.status()).into()),
            created_at: Set(offer.created_at().naive_utc()),
            updated_at: Set(offer.updated_at().naive_utc()),
        }
    }
}
