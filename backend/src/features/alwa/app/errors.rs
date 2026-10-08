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
        ] {
            let info = error.to_error_info();

            assert_eq!(info.kind, kind, "{error:?}");
            assert_eq!(info.code, code, "{error:?}");
        }
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
