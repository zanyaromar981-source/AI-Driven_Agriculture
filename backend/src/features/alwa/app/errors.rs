use thiserror::Error;

use crate::{
    app::{AppError as GlobalAppError, ErrorInfo, ErrorKind, ToErrorInfo},
    features::alwa::domain::AlwaError,
    shared::DomainError,
};

impl ToErrorInfo for AlwaError {
    fn to_error_info(&self) -> ErrorInfo {
        match self {
            AlwaError::ListingNotOpen => {
                ErrorInfo::with_code(ErrorKind::Conflict, "listing_not_open", self.to_string())
            }
            AlwaError::OfferNotOpen => {
                ErrorInfo::with_code(ErrorKind::Conflict, "offer_not_open", self.to_string())
            }
            AlwaError::OwnListing => {
                ErrorInfo::with_code(ErrorKind::InvalidInput, "own_listing", self.to_string())
            }
            AlwaError::OfferTooLarge(_) => {
                ErrorInfo::with_code(ErrorKind::InvalidInput, "offer_too_large", self.to_string())
            }
            AlwaError::BadClosingTime(_) => {
                ErrorInfo::with_code(ErrorKind::InvalidInput, "bad_closes_at", self.to_string())
            }
            AlwaError::AlreadyExists(_) => {
                ErrorInfo::with_code(ErrorKind::Conflict, "already_exists", self.to_string())
            }
            AlwaError::MarketInUse => {
                ErrorInfo::with_code(ErrorKind::Conflict, "market_in_use", self.to_string())
            }
            AlwaError::ListingHasDeal => {
                ErrorInfo::with_code(ErrorKind::Conflict, "listing_has_deal", self.to_string())
            }
            AlwaError::StaffMayOnlyClose => ErrorInfo::with_code(
                ErrorKind::InvalidInput,
                "status_not_allowed",
                self.to_string(),
            ),
            // The web layer adds the `field` to this answer.
            AlwaError::InvalidField { .. } => {
                ErrorInfo::new(ErrorKind::InvalidInput, self.to_string())
            }
            AlwaError::OffersOnlyByKg(_) => {
                ErrorInfo::with_code(ErrorKind::InvalidInput, "offers_kg_only", self.to_string())
            }
            AlwaError::UnknownCrop(_) => {
                ErrorInfo::with_code(ErrorKind::InvalidInput, "unknown_crop", self.to_string())
            }
            // Answered exactly like a listing that does not exist, so the
            // answer does not say whose listing it is.
            AlwaError::NotTheSeller | AlwaError::OfferNotOnListing => {
                GlobalAppError::NotFound.to_error_info()
            }
            AlwaError::DomainError(err) => err.to_error_info(),
        }
    }
}

#[derive(Error, Debug)]
pub enum AppError {
    #[error(transparent)]
    Alwa(#[from] AlwaError),

    #[error(transparent)]
    Domain(#[from] DomainError),

    #[error(transparent)]
    GlobalAppError(#[from] GlobalAppError),

    #[error("You have reached the maximum number of open listings ({0})")]
    MaxOpenListingsReached(u64),
}

impl ToErrorInfo for AppError {
    fn to_error_info(&self) -> ErrorInfo {
        match self {
            AppError::Alwa(err) => err.to_error_info(),
            AppError::Domain(err) => err.to_error_info(),
            AppError::GlobalAppError(err) => err.to_error_info(),
            AppError::MaxOpenListingsReached(_) => ErrorInfo::with_code(
                ErrorKind::InvalidInput,
                "too_many_listings",
                self.to_string(),
            ),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_bad_value_is_invalid_and_an_offer_on_trays_has_its_own_code() {
        let info = AlwaError::InvalidField {
            field: "quantity",
            detail: "Quantity must be 1 to 1000 head".to_string(),
        }
        .to_error_info();

        assert_eq!(info.kind, ErrorKind::InvalidInput);
        assert_eq!(info.code, "invalid");
        assert_eq!(info.detail, "Quantity must be 1 to 1000 head");

        let info = AlwaError::OffersOnlyByKg("head".to_string()).to_error_info();

        assert_eq!(info.kind, ErrorKind::InvalidInput);
        assert_eq!(info.code, "offers_kg_only");
    }

    #[test]
    fn a_broken_rule_carries_the_code_the_app_acts_on() {
        for (error, kind, code) in [
            (
                AlwaError::ListingNotOpen,
                ErrorKind::Conflict,
                "listing_not_open",
            ),
            (
                AlwaError::OfferNotOpen,
                ErrorKind::Conflict,
                "offer_not_open",
            ),
            (
                AlwaError::OwnListing,
                ErrorKind::InvalidInput,
                "own_listing",
            ),
            (
                AlwaError::OfferTooLarge(500),
                ErrorKind::InvalidInput,
                "offer_too_large",
            ),
            (
                AlwaError::BadClosingTime(14),
                ErrorKind::InvalidInput,
                "bad_closes_at",
            ),
            (
                AlwaError::AlreadyExists("market"),
                ErrorKind::Conflict,
                "already_exists",
            ),
            (AlwaError::MarketInUse, ErrorKind::Conflict, "market_in_use"),
            (
                AlwaError::ListingHasDeal,
                ErrorKind::Conflict,
                "listing_has_deal",
            ),
            (
                AlwaError::StaffMayOnlyClose,
                ErrorKind::InvalidInput,
                "status_not_allowed",
            ),
            (
                AlwaError::UnknownCrop("rice".to_string()),
                ErrorKind::InvalidInput,
                "unknown_crop",
            ),
        ] {
            let info = error.to_error_info();

            assert_eq!(info.kind, kind, "{error:?}");
            assert_eq!(info.code, code, "{error:?}");
        }
    }

    #[test]
    fn a_crop_that_cannot_be_used_is_named_in_the_detail() {
        let info = AlwaError::UnknownCrop("rice".to_string()).to_error_info();

        assert!(info.detail.contains("rice"), "{}", info.detail);
    }

    #[test]
    fn someone_elses_listing_looks_like_no_listing_at_all() {
        let missing = GlobalAppError::NotFound.to_error_info();

        assert_eq!(AlwaError::NotTheSeller.to_error_info(), missing);
        assert_eq!(AlwaError::OfferNotOnListing.to_error_info(), missing);
    }

    #[test]
    fn the_listing_quota_has_its_own_code() {
        let info = AppError::MaxOpenListingsReached(20).to_error_info();

        assert_eq!(info.kind, ErrorKind::InvalidInput);
        assert_eq!(info.code, "too_many_listings");
    }
}
