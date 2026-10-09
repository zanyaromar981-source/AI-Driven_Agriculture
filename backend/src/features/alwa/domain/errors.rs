use crate::shared::DomainError;

#[derive(thiserror::Error, Debug)]
pub enum AlwaError {
    #[error("The listing is not open")]
    ListingNotOpen,

    #[error("The offer is not open")]
    OfferNotOpen,

    #[error("You cannot make an offer on your own listing")]
    OwnListing,

    #[error("Only the seller of a listing may do this")]
    NotTheSeller,

    #[error("The offer is not on this listing")]
    OfferNotOnListing,

    #[error("An offer cannot be for more than the {0} kg on sale")]
    OfferTooLarge(i32),

    #[error("A listing must close in the future, at most {0} days ahead")]
    BadClosingTime(i64),

    #[error("This {0} already exists")]
    AlreadyExists(&'static str),

    #[error("The market still has prices or listings")]
    MarketInUse,

    #[error("The listing was sold: a deal cannot be deleted")]
    ListingHasDeal,

    #[error("Staff may only close a listing")]
    StaffMayOnlyClose,

    #[error(transparent)]
    DomainError(#[from] DomainError),
}
