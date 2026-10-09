use chrono::{DateTime, Duration, NaiveDate, Utc};
use getset::Getters;

use crate::{
    features::alwa::domain::{
        AlwaError, BuyerKind, Crop, DisplayName, FairPrice, Grade, ListingStatus, MarketName,
        MarketSlug, Note, OfferStatus, Pickup, PricePerKg, PriceSource, QuantityKg, ZoneSlug,
    },
    shared::Phone,
};

/// A listing stays on the board for two weeks at most.
pub const MAX_CLOSING_DAYS: i64 = 14;

/// How far back the fair price check looks for the alwa's price when there
/// is none on the day the listing was made.
pub const REFERENCE_PRICE_LOOKBACK_DAYS: i64 = 7;

/// The price board compares a price with the one this many days earlier.
pub const PRICE_CHANGE_DAYS: i64 = 7;

pub const MAX_OPEN_LISTINGS_PER_SELLER: u64 = 20;

/// The change from one price to another as a whole percent.
fn percent_change(from: PricePerKg, to: PricePerKg) -> i32 {
    let from = f64::from(from.value());
    let to = f64::from(to.value());

    ((to - from) / from * 100.0).round() as i32
}

/// An alwa: the wholesale produce market of one city. The first ones are
/// seeded by the migration; staff add, rename and remove them on the
/// dashboard.
#[derive(Clone, Debug, PartialEq, Eq, Getters)]
#[getset(get = "pub")]
pub struct Market {
    id: i32,
    slug: MarketSlug,
    name_en: String,
    name_ku: String,
}

impl Market {
    /// Reconstruct from persisted state.
    pub fn rehydrate(id: i32, slug: MarketSlug, name_en: String, name_ku: String) -> Self {
        Self {
            id,
            slug,
            name_en,
            name_ku,
        }
    }
}

/// The two names staff give an alwa. Its slug is fixed once it is made.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MarketNames {
    pub name_en: MarketName,
    pub name_ku: MarketName,
}

/// The price of one crop at one alwa on one day, as a data job reported it
/// or staff entered it.
#[derive(Clone, Debug, Getters)]
#[getset(get = "pub")]
pub struct Price {
    /// None = new (not yet persisted), Some = existing (persisted)
    id: Option<i32>,
    market_id: i32,
    crop: Crop,
    day: NaiveDate,
    price: PricePerKg,
    /// Set by the government rather than by the market, as for wheat.
    fixed: bool,
    source: PriceSource,
    updated_at: DateTime<Utc>,
}

impl Price {
    pub fn new(
        market: &Market,
        crop: Crop,
        day: NaiveDate,
        price: PricePerKg,
        fixed: bool,
        source: PriceSource,
    ) -> Self {
        Self {
            id: None,
            market_id: market.id,
            crop,
            day,
            price,
            fixed,
            source,
            updated_at: Utc::now(),
        }
    }

    /// Reconstruct from persisted state.
    #[allow(clippy::too_many_arguments)]
    pub fn rehydrate(
        id: i32,
        market_id: i32,
        crop: Crop,
        day: NaiveDate,
        price: PricePerKg,
        fixed: bool,
        source: PriceSource,
        updated_at: DateTime<Utc>,
    ) -> Self {
        Self {
            id: Some(id),
            market_id,
            crop,
            day,
            price,
            fixed,
            source,
            updated_at,
        }
    }

    /// The whole percent this price moved from an earlier one. A fixed price
    /// does not move with the market, so it reports no change.
    pub fn change_pct_from(&self, earlier: Option<&Price>) -> Option<i32> {
        if self.fixed {
            return None;
        }

        earlier.map(|earlier| percent_change(earlier.price, self.price))
    }
}

/// What a seller fills in to put a crop on sale.
#[derive(Clone, Debug)]
pub struct ListingDraft {
    pub seller_name: Option<DisplayName>,
    pub crop: Crop,
    pub quantity: QuantityKg,
    pub asking_price: PricePerKg,
    pub grade: Option<Grade>,
    pub pickup: Pickup,
    pub zone_slug: Option<ZoneSlug>,
    pub note: Option<Note>,
    pub closes_at: DateTime<Utc>,
}

/// What a buyer fills in to make an offer.
#[derive(Clone, Debug)]
pub struct OfferDraft {
    pub buyer_name: DisplayName,
    pub buyer_kind: BuyerKind,
    pub quantity: QuantityKg,
    pub price: PricePerKg,
}

/// A crop a farmer has put on sale at an alwa.
#[derive(Clone, Debug, Getters)]
#[getset(get = "pub")]
pub struct Listing {
    /// None = new (not yet persisted), Some = existing (persisted)
    id: Option<i32>,
    /// Private: see `seller_phone_shown_to`.
    seller_phone: Phone,
    seller_name: Option<DisplayName>,
    crop: Crop,
    quantity: QuantityKg,
    asking_price: PricePerKg,
    grade: Option<Grade>,
    pickup: Pickup,
    market_id: i32,
    market: MarketSlug,
    zone_slug: Option<ZoneSlug>,
    note: Option<Note>,
    closes_at: DateTime<Utc>,
    /// The stored status. Read `status_at` instead: a listing whose closing
    /// time has passed is still stored as `Open`.
    status: ListingStatus,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    /// Set once staff closed the listing on the dashboard.
    moderation: Option<Moderation>,
}

/// Who on the staff closed a listing, and the reason they gave.
#[derive(Clone, Debug, PartialEq, Eq, Getters)]
#[getset(get = "pub")]
pub struct Moderation {
    staff_id: i32,
    note: Option<Note>,
}

impl Moderation {
    pub fn new(staff_id: i32, note: Option<Note>) -> Self {
        Self { staff_id, note }
    }
}

impl Listing {
    pub fn new(
        seller: Phone,
        market: &Market,
        draft: ListingDraft,
        now: DateTime<Utc>,
    ) -> Result<Self, AlwaError> {
        if draft.closes_at <= now || draft.closes_at > now + Duration::days(MAX_CLOSING_DAYS) {
            return Err(AlwaError::BadClosingTime(MAX_CLOSING_DAYS));
        }

        Ok(Self {
            id: None,
            seller_phone: seller,
            seller_name: draft.seller_name,
            crop: draft.crop,
            quantity: draft.quantity,
            asking_price: draft.asking_price,
            grade: draft.grade,
            pickup: draft.pickup,
            market_id: market.id,
            market: market.slug.clone(),
            zone_slug: draft.zone_slug,
            note: draft.note,
            closes_at: draft.closes_at,
            status: ListingStatus::Open,
            created_at: now,
            updated_at: now,
            moderation: None,
        })
    }

    /// Reconstruct from persisted state.
    #[allow(clippy::too_many_arguments)]
    pub fn rehydrate(
        id: i32,
        seller_phone: Phone,
        seller_name: Option<DisplayName>,
        crop: Crop,
        quantity: QuantityKg,
        asking_price: PricePerKg,
        grade: Option<Grade>,
        pickup: Pickup,
        market_id: i32,
        market: MarketSlug,
        zone_slug: Option<ZoneSlug>,
        note: Option<Note>,
        closes_at: DateTime<Utc>,
        status: ListingStatus,
        created_at: DateTime<Utc>,
        updated_at: DateTime<Utc>,
    ) -> Self {
        Self {
            id: Some(id),
            seller_phone,
            seller_name,
            crop,
            quantity,
            asking_price,
            grade,
            pickup,
            market_id,
            market,
            zone_slug,
            note,
            closes_at,
            status,
            created_at,
            updated_at,
            moderation: None,
        }
    }

    /// Completes `rehydrate` for a listing stored as closed by staff.
    pub fn moderated(mut self, moderation: Moderation) -> Self {
        self.moderation = Some(moderation);
        self
    }

    /// The status a reader sees. Nothing runs at the closing time, so a
    /// listing that is past it and still stored as open reads as closed.
    pub fn status_at(&self, now: DateTime<Utc>) -> ListingStatus {
        if self.status == ListingStatus::Open && now >= self.closes_at {
            return ListingStatus::Closed;
        }

        self.status
    }

    pub fn is_open_at(&self, now: DateTime<Utc>) -> bool {
        self.status_at(now) == ListingStatus::Open
    }

    pub fn is_sold_by(&self, phone: &Phone) -> bool {
        &self.seller_phone == phone
    }

    /// True when this seller has already taken the listing down. A repeated
    /// cancel then has nothing left to do and is not an error.
    pub fn was_cancelled_by(&self, phone: &Phone) -> bool {
        self.is_sold_by(phone) && self.status == ListingStatus::Cancelled
    }

    /// True when this seller has already made the deal on exactly this
    /// offer. A repeated accept of the same offer then has nothing left to
    /// do and is not an error; accepting a different offer still is.
    pub fn was_sold_on(&self, by: &Phone, offer_id: i32, offers: &[Offer]) -> bool {
        self.is_sold_by(by)
            && self.status == ListingStatus::Sold
            && offers.iter().any(|offer| {
                offer.is_on(self)
                    && offer.id == Some(offer_id)
                    && offer.status == OfferStatus::Accepted
            })
    }

    /// True when staff have already closed the listing with exactly this
    /// note. A repeated close then has nothing left to do and is not an
    /// error; a close with another note still is.
    pub fn was_closed_by_staff_with(&self, note: Option<&Note>) -> bool {
        self.status == ListingStatus::Closed
            && self
                .moderation
                .as_ref()
                .is_some_and(|moderation| moderation.note.as_ref() == note)
    }

    /// A staff member moderates the listing. Closing an open listing is all
    /// they may do: only a deal sells a listing, and nothing reopens one.
    pub fn moderate(
        &mut self,
        to: ListingStatus,
        staff_id: i32,
        note: Option<Note>,
        now: DateTime<Utc>,
    ) -> Result<(), AlwaError> {
        if to != ListingStatus::Closed {
            return Err(AlwaError::StaffMayOnlyClose);
        }

        if !self.is_open_at(now) {
            return Err(AlwaError::ListingNotOpen);
        }

        self.status = ListingStatus::Closed;
        self.updated_at = now;
        self.moderation = Some(Moderation::new(staff_id, note));

        Ok(())
    }

    /// The seller takes the listing down.
    pub fn cancel(&mut self, by: &Phone, now: DateTime<Utc>) -> Result<(), AlwaError> {
        if !self.is_sold_by(by) {
            return Err(AlwaError::NotTheSeller);
        }

        if !self.is_open_at(now) {
            return Err(AlwaError::ListingNotOpen);
        }

        self.status = ListingStatus::Cancelled;
        self.updated_at = now;

        Ok(())
    }

    /// A buyer makes an offer. `existing` are the offers already on this
    /// listing: a buyer has one open offer at most, so an open one they made
    /// earlier is withdrawn in favour of the new one.
    pub fn place_offer(
        &self,
        buyer: Phone,
        draft: OfferDraft,
        existing: &mut [Offer],
        now: DateTime<Utc>,
    ) -> Result<Offer, AlwaError> {
        if self.is_sold_by(&buyer) {
            return Err(AlwaError::OwnListing);
        }

        if !self.is_open_at(now) {
            return Err(AlwaError::ListingNotOpen);
        }

        if draft.quantity > self.quantity {
            return Err(AlwaError::OfferTooLarge(self.quantity.value()));
        }

        for earlier in existing
            .iter_mut()
            .filter(|offer| offer.is_on(self) && offer.is_by(&buyer) && offer.is_open())
        {
            earlier.settle(OfferStatus::Withdrawn, now);
        }

        Ok(Offer {
            id: None,
            listing_id: self.id.unwrap_or_default(),
            buyer_phone: buyer,
            buyer_name: draft.buyer_name,
            buyer_kind: draft.buyer_kind,
            quantity: draft.quantity,
            price: draft.price,
            status: OfferStatus::Open,
            created_at: now,
            updated_at: now,
        })
    }

    /// The seller takes one offer: the listing is sold, that offer is the
    /// deal, and every other open offer in `offers` is declined.
    pub fn accept(
        &mut self,
        by: &Phone,
        offer_id: i32,
        offers: &mut [Offer],
        now: DateTime<Utc>,
    ) -> Result<(), AlwaError> {
        if !self.is_sold_by(by) {
            return Err(AlwaError::NotTheSeller);
        }

        let Some(chosen) = offers
            .iter()
            .position(|offer| offer.is_on(self) && offer.id == Some(offer_id))
        else {
            return Err(AlwaError::OfferNotOnListing);
        };

        if !self.is_open_at(now) {
            return Err(AlwaError::ListingNotOpen);
        }

        if !offers[chosen].is_open() {
            return Err(AlwaError::OfferNotOpen);
        }

        self.status = ListingStatus::Sold;
        self.updated_at = now;

        for (index, offer) in offers.iter_mut().enumerate() {
            if index == chosen {
                offer.settle(OfferStatus::Accepted, now);
            } else if offer.is_on(self) && offer.is_open() {
                offer.settle(OfferStatus::Declined, now);
            }
        }

        Ok(())
    }

    /// The seller's phone is private. Only the buyer whose offer became the
    /// deal is shown it, so the two can arrange the pickup.
    pub fn seller_phone_shown_to(&self, viewer: &Phone, offer: &Offer) -> Option<&Phone> {
        (offer.is_on(self) && offer.is_accepted() && offer.is_by(viewer))
            .then_some(&self.seller_phone)
    }

    /// The alwa's price this listing is measured against: the price of its
    /// crop at its market on the day it was made, or failing that the latest
    /// one in the days just before.
    pub fn reference_price<'a>(&self, prices: &'a [Price]) -> Option<&'a Price> {
        let made_on = self.created_at.date_naive();
        let earliest = made_on - Duration::days(REFERENCE_PRICE_LOOKBACK_DAYS);

        prices
            .iter()
            .filter(|price| price.market_id == self.market_id && price.crop == self.crop)
            .filter(|price| price.day <= made_on && price.day >= earliest)
            .max_by_key(|price| price.day)
    }

    pub fn fair_price(&self, prices: &[Price]) -> FairPrice {
        FairPrice::judge(
            self.asking_price,
            self.reference_price(prices).map(|price| price.price),
        )
    }
}

/// What a buyer is ready to pay for a listing, or for part of it.
#[derive(Clone, Debug, Getters)]
#[getset(get = "pub")]
pub struct Offer {
    /// None = new (not yet persisted), Some = existing (persisted)
    id: Option<i32>,
    listing_id: i32,
    /// Private: see `buyer_phone_shown_to`.
    buyer_phone: Phone,
    buyer_name: DisplayName,
    buyer_kind: BuyerKind,
    quantity: QuantityKg,
    price: PricePerKg,
    status: OfferStatus,
    created_at: DateTime<Utc>,
    /// When the status last changed. For an accepted offer this is the
    /// moment of the deal.
    updated_at: DateTime<Utc>,
}

impl Offer {
    /// Reconstruct from persisted state. A new offer is made through
    /// `Listing::place_offer`, which holds the rules for making one.
    #[allow(clippy::too_many_arguments)]
    pub fn rehydrate(
        id: i32,
        listing_id: i32,
        buyer_phone: Phone,
        buyer_name: DisplayName,
        buyer_kind: BuyerKind,
        quantity: QuantityKg,
        price: PricePerKg,
        status: OfferStatus,
        created_at: DateTime<Utc>,
        updated_at: DateTime<Utc>,
    ) -> Self {
        Self {
            id: Some(id),
            listing_id,
            buyer_phone,
            buyer_name,
            buyer_kind,
            quantity,
            price,
            status,
            created_at,
            updated_at,
        }
    }

    pub fn is_open(&self) -> bool {
        self.status == OfferStatus::Open
    }

    pub fn is_accepted(&self) -> bool {
        self.status == OfferStatus::Accepted
    }

    pub fn is_by(&self, phone: &Phone) -> bool {
        &self.buyer_phone == phone
    }

    pub fn is_on(&self, listing: &Listing) -> bool {
        listing.id == Some(self.listing_id)
    }

    /// The buyer's phone is private. Only the seller is shown it, and only
    /// on the offer that became the deal.
    pub fn buyer_phone_shown_to(&self, viewer: &Phone, listing: &Listing) -> Option<&Phone> {
        (self.is_on(listing) && self.is_accepted() && listing.is_sold_by(viewer))
            .then_some(&self.buyer_phone)
    }

    fn settle(&mut self, status: OfferStatus, now: DateTime<Utc>) {
        self.status = status;
        self.updated_at = now;
    }
}

/// A listing as the board shows it: with the status a reader sees, its
/// offers and the fair price check.
#[derive(Clone, Debug, Getters)]
#[getset(get = "pub")]
pub struct ListingCard {
    listing: Listing,
    status: ListingStatus,
    /// Highest price first, as a seller reads them.
    offers: Vec<Offer>,
    fair_price: FairPrice,
}

impl ListingCard {
    /// `offers` and `prices` may hold rows of other listings too, as they do
    /// when a whole page is loaded at once: the card takes only its own.
    pub fn assemble(
        listing: Listing,
        offers: &[Offer],
        prices: &[Price],
        now: DateTime<Utc>,
    ) -> Self {
        let mut offers: Vec<Offer> = offers
            .iter()
            .filter(|offer| offer.is_on(&listing))
            .cloned()
            .collect();

        // Between two equal prices the earlier offer leads.
        offers.sort_by(|first, second| {
            second
                .price
                .cmp(&first.price)
                .then(first.created_at.cmp(&second.created_at))
        });

        Self {
            status: listing.status_at(now),
            fair_price: listing.fair_price(prices),
            listing,
            offers,
        }
    }

    pub fn open_offers(&self) -> usize {
        self.offers.iter().filter(|offer| offer.is_open()).count()
    }

    pub fn best_offer(&self) -> Option<PricePerKg> {
        self.offers
            .iter()
            .filter(|offer| offer.is_open())
            .map(|offer| offer.price)
            .max()
    }
}

/// One of a buyer's own offers with the listing it was made on.
#[derive(Clone, Debug, Getters)]
#[getset(get = "pub")]
pub struct PlacedOffer {
    offer: Offer,
    listing: Listing,
    listing_status: ListingStatus,
}

impl PlacedOffer {
    pub fn new(offer: Offer, listing: Listing, now: DateTime<Utc>) -> Self {
        Self {
            listing_status: listing.status_at(now),
            offer,
            listing,
        }
    }
}

/// A sale: an accepted offer and the listing it was made on. There is no
/// deal of its own in the database.
#[derive(Clone, Debug, Getters)]
#[getset(get = "pub")]
pub struct Deal {
    listing: Listing,
    offer: Offer,
}

impl Deal {
    /// `None` unless the offer is on this listing and was accepted.
    pub fn new(listing: Listing, offer: Offer) -> Option<Self> {
        (offer.is_on(&listing) && offer.is_accepted()).then_some(Self { listing, offer })
    }

    pub fn accepted_at(&self) -> DateTime<Utc> {
        self.offer.updated_at
    }

    /// How far the sold price is from the asking price, as a whole percent.
    /// Negative when the crop went for less than was asked.
    pub fn vs_asking_pct(&self) -> i32 {
        percent_change(self.listing.asking_price, self.offer.price)
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DealsSummary {
    pub deals: usize,
    pub tonnes: f64,
}

impl DealsSummary {
    pub fn of(deals: &[Deal]) -> Self {
        let kilograms: i64 = deals
            .iter()
            .map(|deal| i64::from(deal.offer.quantity.value()))
            .sum();

        Self {
            deals: deals.len(),
            tonnes: kilograms as f64 / 1_000.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SELLER: &str = "+9647501234567";
    const BUYER: &str = "+9647701112233";
    const OTHER_BUYER: &str = "+9647709998877";

    fn phone(value: &str) -> Phone {
        Phone::new(value.to_string()).expect("phone")
    }

    fn market() -> Market {
        Market::rehydrate(
            1,
            MarketSlug::new("sulaymaniyah".to_string()).expect("slug"),
            "Sulaymaniyah".to_string(),
            "سلێمانی".to_string(),
        )
    }

    fn draft(closes_at: DateTime<Utc>) -> ListingDraft {
        ListingDraft {
            seller_name: Some(DisplayName::new("Kak Azad".to_string()).expect("name")),
            crop: Crop::Tomato,
            quantity: QuantityKg::new(500).expect("quantity"),
            asking_price: PricePerKg::new(1_000).expect("price"),
            grade: Some(Grade::A),
            pickup: Pickup::Farm,
            zone_slug: None,
            note: None,
            closes_at,
        }
    }

    /// A persisted open listing with id 7: 500 kg of tomato at 1,000 IQD,
    /// closing three days after `now`.
    fn listing(now: DateTime<Utc>) -> Listing {
        let mut listing = Listing::new(
            phone(SELLER),
            &market(),
            draft(now + Duration::days(3)),
            now,
        )
        .expect("listing");
        listing.id = Some(7);

        listing
    }

    fn offer_draft(quantity: i64, price: i64) -> OfferDraft {
        OfferDraft {
            buyer_name: DisplayName::new("Bazaar shop".to_string()).expect("name"),
            buyer_kind: BuyerKind::Shop,
            quantity: QuantityKg::new(quantity).expect("quantity"),
            price: PricePerKg::new(price).expect("price"),
        }
    }

    /// A persisted open offer on listing 7.
    fn offer(id: i32, buyer: &str, price: i64, now: DateTime<Utc>) -> Offer {
        let mut offer = listing(now)
            .place_offer(phone(buyer), offer_draft(500, price), &mut [], now)
            .expect("offer");
        offer.id = Some(id);

        offer
    }

    fn price_on(day: NaiveDate, value: i64, fixed: bool) -> Price {
        Price::rehydrate(
            1,
            1,
            Crop::Tomato,
            day,
            PricePerKg::new(value).expect("price"),
            fixed,
            PriceSource::new("alwa-board".to_string()).expect("source"),
            Utc::now(),
        )
    }

    #[test]
    fn a_new_listing_is_open_and_belongs_to_its_market() {
        let now = Utc::now();
        let listing = Listing::new(
            phone(SELLER),
            &market(),
            draft(now + Duration::days(3)),
            now,
        )
        .expect("listing");

        assert_eq!(*listing.status(), ListingStatus::Open);
        assert_eq!(*listing.market_id(), 1);
        assert_eq!(listing.market().as_str(), "sulaymaniyah");
        assert!(listing.id().is_none());
    }

    #[test]
    fn a_listing_must_close_in_the_future() {
        let now = Utc::now();

        for closes_at in [now, now - Duration::hours(1)] {
            assert!(matches!(
                Listing::new(phone(SELLER), &market(), draft(closes_at), now),
                Err(AlwaError::BadClosingTime(_))
            ));
        }
    }

    #[test]
    fn a_listing_closes_within_fourteen_days_at_most() {
        let now = Utc::now();
        let last_moment = now + Duration::days(MAX_CLOSING_DAYS);

        assert!(Listing::new(phone(SELLER), &market(), draft(last_moment), now).is_ok());
        assert!(matches!(
            Listing::new(
                phone(SELLER),
                &market(),
                draft(last_moment + Duration::seconds(1)),
                now
            ),
            Err(AlwaError::BadClosingTime(MAX_CLOSING_DAYS))
        ));
    }

    #[test]
    fn an_open_listing_past_its_closing_time_reads_as_closed() {
        let now = Utc::now();
        let listing = listing(now);

        assert_eq!(listing.status_at(now), ListingStatus::Open);
        assert_eq!(
            listing.status_at(*listing.closes_at()),
            ListingStatus::Closed,
            "the closing time itself is already closed"
        );
        assert_eq!(
            *listing.status(),
            ListingStatus::Open,
            "reading never changes what is stored"
        );
    }

    #[test]
    fn a_sold_listing_stays_sold_after_its_closing_time() {
        let now = Utc::now();
        let mut listing = listing(now);
        let mut offers = [offer(1, BUYER, 950, now)];

        listing
            .accept(&phone(SELLER), 1, &mut offers, now)
            .expect("accept");

        assert_eq!(
            listing.status_at(now + Duration::days(30)),
            ListingStatus::Sold
        );
    }

    #[test]
    fn only_the_seller_may_cancel_a_listing() {
        let now = Utc::now();
        let mut listing = listing(now);

        assert!(matches!(
            listing.cancel(&phone(BUYER), now),
            Err(AlwaError::NotTheSeller)
        ));
        assert_eq!(*listing.status(), ListingStatus::Open);

        listing.cancel(&phone(SELLER), now).expect("cancel");
        assert_eq!(*listing.status(), ListingStatus::Cancelled);
    }

    #[test]
    fn a_listing_that_is_not_open_cannot_be_cancelled() {
        let now = Utc::now();
        let mut cancelled = listing(now);
        cancelled.cancel(&phone(SELLER), now).expect("cancel");

        assert!(matches!(
            cancelled.cancel(&phone(SELLER), now),
            Err(AlwaError::ListingNotOpen)
        ));

        let mut expired = listing(now);

        assert!(matches!(
            expired.cancel(&phone(SELLER), now + Duration::days(4)),
            Err(AlwaError::ListingNotOpen)
        ));
    }

    #[test]
    fn staff_close_an_open_listing_and_leave_their_name_on_it() {
        let now = Utc::now();
        let mut listing = listing(now);
        let note = Note::new("duplicate".to_string()).expect("note");

        listing
            .moderate(ListingStatus::Closed, 9, Some(note.clone()), now)
            .expect("close");

        assert_eq!(*listing.status(), ListingStatus::Closed);
        assert_eq!(
            listing.moderation(),
            &Some(Moderation::new(9, Some(note.clone())))
        );
        assert!(listing.was_closed_by_staff_with(Some(&note)));
        assert!(
            !listing.was_closed_by_staff_with(None),
            "a close with another note is a different request"
        );
    }

    #[test]
    fn staff_may_not_sell_reopen_or_cancel_a_listing() {
        let now = Utc::now();

        for to in [
            ListingStatus::Open,
            ListingStatus::Sold,
            ListingStatus::Cancelled,
        ] {
            let mut listing = listing(now);

            assert!(matches!(
                listing.moderate(to, 9, None, now),
                Err(AlwaError::StaffMayOnlyClose)
            ));
            assert_eq!(*listing.status(), ListingStatus::Open);
        }
    }

    #[test]
    fn staff_cannot_close_a_listing_that_is_not_open() {
        let now = Utc::now();
        let mut cancelled = listing(now);
        cancelled.cancel(&phone(SELLER), now).expect("cancel");
        let mut expired = listing(now - Duration::days(5));

        for listing in [&mut cancelled, &mut expired] {
            assert!(matches!(
                listing.moderate(ListingStatus::Closed, 9, None, now),
                Err(AlwaError::ListingNotOpen)
            ));
            assert!(listing.moderation().is_none());
        }

        assert!(
            !expired.was_closed_by_staff_with(None),
            "a listing whose time ran out was not closed by staff"
        );
    }

    #[test]
    fn a_user_may_not_offer_on_their_own_listing() {
        let now = Utc::now();

        assert!(matches!(
            listing(now).place_offer(phone(SELLER), offer_draft(100, 900), &mut [], now),
            Err(AlwaError::OwnListing)
        ));
    }

    #[test]
    fn an_offer_on_an_open_listing_starts_open() {
        let now = Utc::now();
        let offer = listing(now)
            .place_offer(phone(BUYER), offer_draft(100, 900), &mut [], now)
            .expect("offer");

        assert!(offer.is_open());
        assert_eq!(*offer.listing_id(), 7);
        assert!(offer.is_by(&phone(BUYER)));
    }

    #[test]
    fn no_offer_can_be_made_once_the_closing_time_has_passed() {
        let now = Utc::now();
        let listing = listing(now);

        assert!(matches!(
            listing.place_offer(
                phone(BUYER),
                offer_draft(100, 900),
                &mut [],
                *listing.closes_at()
            ),
            Err(AlwaError::ListingNotOpen)
        ));
    }

    #[test]
    fn no_offer_can_be_made_on_a_cancelled_or_sold_listing() {
        let now = Utc::now();
        let mut cancelled = listing(now);
        cancelled.cancel(&phone(SELLER), now).expect("cancel");

        assert!(matches!(
            cancelled.place_offer(phone(BUYER), offer_draft(100, 900), &mut [], now),
            Err(AlwaError::ListingNotOpen)
        ));

        let mut sold = listing(now);
        let mut offers = [offer(1, BUYER, 950, now)];
        sold.accept(&phone(SELLER), 1, &mut offers, now)
            .expect("accept");

        assert!(matches!(
            sold.place_offer(phone(OTHER_BUYER), offer_draft(100, 900), &mut [], now),
            Err(AlwaError::ListingNotOpen)
        ));
    }

    #[test]
    fn an_offer_cannot_be_for_more_than_is_on_sale() {
        let now = Utc::now();
        let listing = listing(now);

        assert!(
            listing
                .place_offer(phone(BUYER), offer_draft(500, 900), &mut [], now)
                .is_ok(),
            "the whole lot can be offered for"
        );
        assert!(matches!(
            listing.place_offer(phone(BUYER), offer_draft(501, 900), &mut [], now),
            Err(AlwaError::OfferTooLarge(500))
        ));
    }

    #[test]
    fn a_new_offer_replaces_the_buyers_previous_open_one() {
        let now = Utc::now();
        let mut existing = [offer(1, BUYER, 900, now), offer(2, OTHER_BUYER, 920, now)];

        let newer = listing(now)
            .place_offer(phone(BUYER), offer_draft(500, 950), &mut existing, now)
            .expect("offer");

        assert!(newer.is_open());
        assert_eq!(*existing[0].status(), OfferStatus::Withdrawn);
        assert_eq!(
            *existing[1].status(),
            OfferStatus::Open,
            "another buyer's offer is not touched"
        );
    }

    #[test]
    fn a_refused_offer_withdraws_nothing() {
        let now = Utc::now();
        let mut existing = [offer(1, BUYER, 900, now)];

        let result =
            listing(now).place_offer(phone(BUYER), offer_draft(501, 950), &mut existing, now);

        assert!(result.is_err());
        assert!(existing[0].is_open());
    }

    #[test]
    fn accepting_sells_the_listing_and_declines_every_other_open_offer() {
        let now = Utc::now();
        let mut listing = listing(now);
        let mut offers = [
            offer(1, BUYER, 950, now),
            offer(2, OTHER_BUYER, 900, now),
            offer(3, OTHER_BUYER, 800, now),
        ];
        offers[2].settle(OfferStatus::Withdrawn, now);

        listing
            .accept(&phone(SELLER), 1, &mut offers, now)
            .expect("accept");

        assert_eq!(*listing.status(), ListingStatus::Sold);
        assert_eq!(*offers[0].status(), OfferStatus::Accepted);
        assert_eq!(*offers[1].status(), OfferStatus::Declined);
        assert_eq!(
            *offers[2].status(),
            OfferStatus::Withdrawn,
            "an offer that was no longer open keeps its status"
        );
    }

    #[test]
    fn only_the_seller_may_accept_an_offer() {
        let now = Utc::now();
        let mut listing = listing(now);
        let mut offers = [offer(1, BUYER, 950, now)];

        assert!(matches!(
            listing.accept(&phone(BUYER), 1, &mut offers, now),
            Err(AlwaError::NotTheSeller)
        ));
        assert_eq!(*listing.status(), ListingStatus::Open);
        assert!(offers[0].is_open());
    }

    #[test]
    fn an_offer_that_is_not_on_the_listing_cannot_be_accepted() {
        let now = Utc::now();
        let mut listing = listing(now);
        let mut offers = [offer(1, BUYER, 950, now)];

        assert!(matches!(
            listing.accept(&phone(SELLER), 99, &mut offers, now),
            Err(AlwaError::OfferNotOnListing)
        ));
    }

    #[test]
    fn a_listing_that_is_not_open_cannot_accept() {
        let now = Utc::now();
        let mut offers = [offer(1, BUYER, 950, now), offer(2, OTHER_BUYER, 900, now)];

        let mut sold = listing(now);
        sold.accept(&phone(SELLER), 1, &mut offers, now)
            .expect("accept");

        assert!(matches!(
            sold.accept(&phone(SELLER), 2, &mut offers, now),
            Err(AlwaError::ListingNotOpen)
        ));

        let mut expired = listing(now);
        let mut offers = [offer(1, BUYER, 950, now)];

        assert!(matches!(
            expired.accept(&phone(SELLER), 1, &mut offers, now + Duration::days(4)),
            Err(AlwaError::ListingNotOpen)
        ));
        assert!(offers[0].is_open());
    }

    #[test]
    fn an_offer_that_is_not_open_cannot_be_accepted() {
        let now = Utc::now();
        let mut listing = listing(now);
        let mut offers = [offer(1, BUYER, 950, now)];
        offers[0].settle(OfferStatus::Withdrawn, now);

        assert!(matches!(
            listing.accept(&phone(SELLER), 1, &mut offers, now),
            Err(AlwaError::OfferNotOpen)
        ));
        assert_eq!(*listing.status(), ListingStatus::Open);
    }

    #[test]
    fn phones_stay_private_while_there_is_no_deal() {
        let now = Utc::now();
        let listing = listing(now);
        let offer = offer(1, BUYER, 950, now);

        assert!(
            offer
                .buyer_phone_shown_to(&phone(SELLER), &listing)
                .is_none()
        );
        assert!(
            listing
                .seller_phone_shown_to(&phone(BUYER), &offer)
                .is_none()
        );
    }

    #[test]
    fn after_a_deal_the_seller_and_the_accepted_buyer_see_each_others_phone() {
        let now = Utc::now();
        let mut listing = listing(now);
        let mut offers = [offer(1, BUYER, 950, now), offer(2, OTHER_BUYER, 900, now)];

        listing
            .accept(&phone(SELLER), 1, &mut offers, now)
            .expect("accept");

        assert_eq!(
            offers[0]
                .buyer_phone_shown_to(&phone(SELLER), &listing)
                .map(Phone::as_str),
            Some(BUYER)
        );
        assert_eq!(
            listing
                .seller_phone_shown_to(&phone(BUYER), &offers[0])
                .map(Phone::as_str),
            Some(SELLER)
        );
    }

    #[test]
    fn a_deal_shows_no_phone_to_anyone_else() {
        let now = Utc::now();
        let mut listing = listing(now);
        let mut offers = [offer(1, BUYER, 950, now), offer(2, OTHER_BUYER, 900, now)];

        listing
            .accept(&phone(SELLER), 1, &mut offers, now)
            .expect("accept");

        assert!(
            offers[0]
                .buyer_phone_shown_to(&phone(OTHER_BUYER), &listing)
                .is_none(),
            "a stranger never sees the buyer's phone"
        );
        assert!(
            offers[1]
                .buyer_phone_shown_to(&phone(SELLER), &listing)
                .is_none(),
            "the seller does not see the phone behind a declined offer"
        );
        assert!(
            listing
                .seller_phone_shown_to(&phone(OTHER_BUYER), &offers[1])
                .is_none(),
            "a declined buyer does not see the seller's phone"
        );
        assert!(
            listing
                .seller_phone_shown_to(&phone(OTHER_BUYER), &offers[0])
                .is_none(),
            "another buyer cannot borrow the accepted offer"
        );
    }

    #[test]
    fn the_fair_price_check_uses_the_price_on_the_day_the_listing_was_made() {
        let now = Utc::now();
        let listing = listing(now);
        let today = now.date_naive();

        let prices = [
            price_on(today - Duration::days(1), 500, false),
            price_on(today, 1_050, false),
        ];

        assert_eq!(listing.fair_price(&prices), FairPrice::Fair);
        assert_eq!(
            listing.fair_price(&[price_on(today, 800, false)]),
            FairPrice::High
        );
        assert_eq!(
            listing.fair_price(&[price_on(today, 1_500, false)]),
            FairPrice::Low
        );
    }

    #[test]
    fn without_a_price_that_day_the_latest_one_of_the_week_before_is_used() {
        let now = Utc::now();
        let listing = listing(now);
        let today = now.date_naive();

        let prices = [
            price_on(today - Duration::days(6), 2_000, false),
            price_on(today - Duration::days(2), 1_000, false),
        ];

        assert_eq!(
            listing.reference_price(&prices).map(|price| *price.day()),
            Some(today - Duration::days(2))
        );
        assert_eq!(listing.fair_price(&prices), FairPrice::Fair);
    }

    #[test]
    fn a_price_older_than_seven_days_or_from_a_later_day_is_not_a_reference() {
        let now = Utc::now();
        let listing = listing(now);
        let today = now.date_naive();

        assert!(
            listing
                .reference_price(&[price_on(today - Duration::days(7), 1_000, false)])
                .is_some(),
            "seven days back is still inside the window"
        );
        assert_eq!(
            listing.fair_price(&[
                price_on(today - Duration::days(8), 1_000, false),
                price_on(today + Duration::days(1), 1_000, false),
            ]),
            FairPrice::Unknown
        );
    }

    #[test]
    fn a_price_of_another_crop_or_market_is_not_a_reference() {
        let now = Utc::now();
        let listing = listing(now);
        let today = now.date_naive();
        let source = || PriceSource::new("alwa-board".to_string()).expect("source");
        let price = PricePerKg::new(1_000).expect("price");

        let prices = [
            Price::rehydrate(1, 1, Crop::Onion, today, price, false, source(), now),
            Price::rehydrate(2, 2, Crop::Tomato, today, price, false, source(), now),
        ];

        assert_eq!(listing.fair_price(&prices), FairPrice::Unknown);
    }

    #[test]
    fn the_price_change_is_a_whole_percent_against_the_earlier_price() {
        let day = NaiveDate::from_ymd_opt(2026, 10, 8).expect("day");
        let earlier = price_on(day - Duration::days(7), 1_000, false);

        assert_eq!(
            price_on(day, 1_250, false).change_pct_from(Some(&earlier)),
            Some(25)
        );
        assert_eq!(
            price_on(day, 900, false).change_pct_from(Some(&earlier)),
            Some(-10)
        );
        assert_eq!(
            price_on(day, 1_004, false).change_pct_from(Some(&earlier)),
            Some(0),
            "0.4% rounds to nothing"
        );
        assert_eq!(
            price_on(day, 1_005, false).change_pct_from(Some(&earlier)),
            Some(1),
            "half a percent rounds away from zero"
        );
    }

    #[test]
    fn the_price_change_is_missing_without_an_earlier_price_or_for_a_fixed_price() {
        let day = NaiveDate::from_ymd_opt(2026, 10, 8).expect("day");
        let earlier = price_on(day - Duration::days(7), 1_000, false);

        assert_eq!(price_on(day, 1_250, false).change_pct_from(None), None);
        assert_eq!(
            price_on(day, 1_250, true).change_pct_from(Some(&earlier)),
            None,
            "a government price does not move with the market"
        );
    }

    #[test]
    fn a_card_counts_open_offers_and_leads_with_the_highest_price() {
        let now = Utc::now();
        let mut offers = vec![
            offer(1, BUYER, 900, now),
            offer(2, OTHER_BUYER, 980, now),
            offer(3, OTHER_BUYER, 990, now),
        ];
        offers[2].settle(OfferStatus::Withdrawn, now);

        // An offer on some other listing of the same page.
        let mut stray = offer(4, BUYER, 5_000, now);
        stray.listing_id = 8;
        offers.push(stray);

        let card = ListingCard::assemble(listing(now), &offers, &[], now);

        assert_eq!(
            card.offers()
                .iter()
                .map(|offer| offer.id().unwrap_or_default())
                .collect::<Vec<_>>(),
            vec![3, 2, 1]
        );
        assert_eq!(card.open_offers(), 2);
        assert_eq!(
            card.best_offer().map(|price| price.value()),
            Some(980),
            "a withdrawn offer is not the best offer, however high"
        );
        assert_eq!(*card.fair_price(), FairPrice::Unknown);
    }

    #[test]
    fn a_card_without_open_offers_has_no_best_offer() {
        let now = Utc::now();
        let card = ListingCard::assemble(listing(now), &[], &[], now);

        assert_eq!(card.open_offers(), 0);
        assert!(card.best_offer().is_none());
    }

    #[test]
    fn a_card_shows_the_status_a_reader_sees() {
        let now = Utc::now();
        let card = ListingCard::assemble(listing(now), &[], &[], now + Duration::days(4));

        assert_eq!(*card.status(), ListingStatus::Closed);
    }

    #[test]
    fn a_deal_is_an_accepted_offer_and_nothing_else() {
        let now = Utc::now();
        let mut listing = listing(now);
        let mut offers = [offer(1, BUYER, 950, now), offer(2, OTHER_BUYER, 900, now)];

        assert!(Deal::new(listing.clone(), offers[0].clone()).is_none());

        listing
            .accept(&phone(SELLER), 1, &mut offers, now)
            .expect("accept");

        let deal = Deal::new(listing.clone(), offers[0].clone()).expect("deal");

        assert_eq!(deal.accepted_at(), now);
        assert_eq!(deal.vs_asking_pct(), -5);
        assert!(
            Deal::new(listing, offers[1].clone()).is_none(),
            "a declined offer is not a deal"
        );
    }

    #[test]
    fn the_summary_adds_up_the_sold_weight_in_tonnes() {
        let now = Utc::now();
        let mut listing = listing(now);
        let mut offers = [offer(1, BUYER, 950, now)];

        listing
            .accept(&phone(SELLER), 1, &mut offers, now)
            .expect("accept");

        let deal = Deal::new(listing, offers[0].clone()).expect("deal");

        assert_eq!(
            DealsSummary::of(&[deal.clone(), deal]),
            DealsSummary {
                deals: 2,
                tonnes: 1.0
            }
        );
        assert_eq!(
            DealsSummary::of(&[]),
            DealsSummary {
                deals: 0,
                tonnes: 0.0
            }
        );
    }
}
