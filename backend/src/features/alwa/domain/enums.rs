use crate::{
    features::alwa::domain::{AlwaError, PricePerKg},
    shared::DomainError,
};

/// The quality grade a seller gives their produce. `A` is the best.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Grade {
    A,
    B,
    C,
}

impl Grade {
    pub const ALL: [Grade; 3] = [Grade::A, Grade::B, Grade::C];
}

impl From<Grade> for String {
    fn from(value: Grade) -> Self {
        match value {
            Grade::A => "a".to_string(),
            Grade::B => "b".to_string(),
            Grade::C => "c".to_string(),
        }
    }
}

impl TryFrom<&str> for Grade {
    type Error = AlwaError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "a" => Ok(Grade::A),
            "b" => Ok(Grade::B),
            "c" => Ok(Grade::C),
            _ => Err(DomainError::InvalidValue(format!("Invalid grade: {value}")).into()),
        }
    }
}

/// Where the buyer collects the crop: at the farm, or at the alwa itself.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pickup {
    Farm,
    Alwa,
}

impl Pickup {
    pub const ALL: [Pickup; 2] = [Pickup::Farm, Pickup::Alwa];
}

impl From<Pickup> for String {
    fn from(value: Pickup) -> Self {
        match value {
            Pickup::Farm => "farm".to_string(),
            Pickup::Alwa => "alwa".to_string(),
        }
    }
}

impl TryFrom<&str> for Pickup {
    type Error = AlwaError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "farm" => Ok(Pickup::Farm),
            "alwa" => Ok(Pickup::Alwa),
            _ => Err(DomainError::InvalidValue(format!("Invalid pickup: {value}")).into()),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ListingStatus {
    Open,
    Sold,
    /// Its closing time passed with no deal, or staff closed it.
    Closed,
    /// The seller took it down.
    Cancelled,
}

impl ListingStatus {
    pub const ALL: [ListingStatus; 4] = [
        ListingStatus::Open,
        ListingStatus::Sold,
        ListingStatus::Closed,
        ListingStatus::Cancelled,
    ];
}

impl From<ListingStatus> for String {
    fn from(value: ListingStatus) -> Self {
        match value {
            ListingStatus::Open => "open".to_string(),
            ListingStatus::Sold => "sold".to_string(),
            ListingStatus::Closed => "closed".to_string(),
            ListingStatus::Cancelled => "cancelled".to_string(),
        }
    }
}

impl TryFrom<&str> for ListingStatus {
    type Error = AlwaError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "open" => Ok(ListingStatus::Open),
            "sold" => Ok(ListingStatus::Sold),
            "closed" => Ok(ListingStatus::Closed),
            "cancelled" => Ok(ListingStatus::Cancelled),
            _ => Err(DomainError::InvalidValue(format!("Invalid listing status: {value}")).into()),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OfferStatus {
    Open,
    /// The seller took it: this is the deal.
    Accepted,
    /// The seller took another offer, or staff closed the listing.
    Declined,
    /// The buyer replaced it with a newer offer.
    Withdrawn,
}

impl OfferStatus {
    pub const ALL: [OfferStatus; 4] = [
        OfferStatus::Open,
        OfferStatus::Accepted,
        OfferStatus::Declined,
        OfferStatus::Withdrawn,
    ];
}

impl From<OfferStatus> for String {
    fn from(value: OfferStatus) -> Self {
        match value {
            OfferStatus::Open => "open".to_string(),
            OfferStatus::Accepted => "accepted".to_string(),
            OfferStatus::Declined => "declined".to_string(),
            OfferStatus::Withdrawn => "withdrawn".to_string(),
        }
    }
}

impl TryFrom<&str> for OfferStatus {
    type Error = AlwaError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "open" => Ok(OfferStatus::Open),
            "accepted" => Ok(OfferStatus::Accepted),
            "declined" => Ok(OfferStatus::Declined),
            "withdrawn" => Ok(OfferStatus::Withdrawn),
            _ => Err(DomainError::InvalidValue(format!("Invalid offer status: {value}")).into()),
        }
    }
}

/// What kind of buyer stands behind an offer.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BuyerKind {
    Shop,
    Restaurant,
    Trader,
    Other,
}

impl BuyerKind {
    pub const ALL: [BuyerKind; 4] = [
        BuyerKind::Shop,
        BuyerKind::Restaurant,
        BuyerKind::Trader,
        BuyerKind::Other,
    ];
}

impl From<BuyerKind> for String {
    fn from(value: BuyerKind) -> Self {
        match value {
            BuyerKind::Shop => "shop".to_string(),
            BuyerKind::Restaurant => "restaurant".to_string(),
            BuyerKind::Trader => "trader".to_string(),
            BuyerKind::Other => "other".to_string(),
        }
    }
}

impl TryFrom<&str> for BuyerKind {
    type Error = AlwaError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "shop" => Ok(BuyerKind::Shop),
            "restaurant" => Ok(BuyerKind::Restaurant),
            "trader" => Ok(BuyerKind::Trader),
            "other" => Ok(BuyerKind::Other),
            _ => Err(DomainError::InvalidValue(format!("Invalid buyer kind: {value}")).into()),
        }
    }
}

/// How an asking price sits against the alwa's own price for the crop. It is
/// worked out on every read and never stored.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FairPrice {
    Fair,
    High,
    Low,
    /// The alwa had no price for the crop around the day of the listing.
    Unknown,
}

impl FairPrice {
    /// `Fair` within 10% of the reference either way, the edges included.
    /// Whole numbers only, so the edge does not depend on rounding.
    pub fn judge(asking: PricePerKg, reference: Option<PricePerKg>) -> Self {
        let Some(reference) = reference else {
            return FairPrice::Unknown;
        };

        let asking = i64::from(asking.value());
        let reference = i64::from(reference.value());

        if (asking - reference).abs() * 10 <= reference {
            FairPrice::Fair
        } else if asking > reference {
            FairPrice::High
        } else {
            FairPrice::Low
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_grade_round_trips() {
        for grade in Grade::ALL {
            let stored = String::from(grade);

            assert_eq!(Grade::try_from(stored.as_str()).expect("grade"), grade);
        }

        assert!(Grade::try_from("d").is_err());
        assert!(Grade::try_from("A").is_err());
    }

    #[test]
    fn every_pickup_round_trips() {
        for pickup in Pickup::ALL {
            let stored = String::from(pickup);

            assert_eq!(Pickup::try_from(stored.as_str()).expect("pickup"), pickup);
        }

        assert!(Pickup::try_from("home").is_err());
    }

    #[test]
    fn every_listing_status_round_trips() {
        for status in ListingStatus::ALL {
            let stored = String::from(status);

            assert_eq!(
                ListingStatus::try_from(stored.as_str()).expect("status"),
                status
            );
        }

        assert!(ListingStatus::try_from("accepted").is_err());
    }

    #[test]
    fn every_offer_status_round_trips() {
        for status in OfferStatus::ALL {
            let stored = String::from(status);

            assert_eq!(
                OfferStatus::try_from(stored.as_str()).expect("status"),
                status
            );
        }

        assert!(OfferStatus::try_from("sold").is_err());
    }

    #[test]
    fn every_buyer_kind_round_trips() {
        for kind in BuyerKind::ALL {
            let stored = String::from(kind);

            assert_eq!(BuyerKind::try_from(stored.as_str()).expect("kind"), kind);
        }

        assert!(BuyerKind::try_from("farmer").is_err());
    }

    fn price(value: i64) -> PricePerKg {
        PricePerKg::new(value).expect("price")
    }

    #[test]
    fn an_asking_price_within_ten_percent_either_way_is_fair() {
        assert_eq!(
            FairPrice::judge(price(1_000), Some(price(1_000))),
            FairPrice::Fair
        );
        assert_eq!(
            FairPrice::judge(price(1_100), Some(price(1_000))),
            FairPrice::Fair,
            "exactly 10% above is still fair"
        );
        assert_eq!(
            FairPrice::judge(price(900), Some(price(1_000))),
            FairPrice::Fair,
            "exactly 10% below is still fair"
        );
    }

    #[test]
    fn an_asking_price_more_than_ten_percent_off_is_high_or_low() {
        assert_eq!(
            FairPrice::judge(price(1_101), Some(price(1_000))),
            FairPrice::High
        );
        assert_eq!(
            FairPrice::judge(price(899), Some(price(1_000))),
            FairPrice::Low
        );
    }

    #[test]
    fn without_a_price_at_the_alwa_nothing_is_claimed() {
        assert_eq!(FairPrice::judge(price(1_000), None), FairPrice::Unknown);
    }
}
