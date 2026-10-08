use std::sync::Arc;

use chrono::Utc;

use crate::{
    app::{AppError as GlobalAppError, AuthContext},
    features::alwa::app::{AlwaRepository, AppError},
};

pub struct CancelListingUseCase {
    repository: Arc<dyn AlwaRepository>,
}

impl CancelListingUseCase {
    pub fn new(repository: Arc<dyn AlwaRepository>) -> Self {
        Self { repository }
    }

    pub async fn execute(&self, auth_context: &AuthContext, id: i32) -> Result<(), AppError> {
        let Some(mut listing) = self.repository.find_listing_by_id(id).await? else {
            tracing::info!(listing_id = id, "cancel refused: no such listing");

            return Err(GlobalAppError::NotFound.into());
        };

        listing
            .cancel(auth_context.user().phone(), Utc::now())
            .inspect_err(|error| tracing::info!(listing_id = id, %error, "cancel refused"))?;

        self.repository.cancel_listing(&listing).await?;

        tracing::info!(listing_id = id, "listing cancelled");

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use chrono::Duration;

    use super::*;
    use crate::features::alwa::{
        app::testing::{
            BUYER, FakeAlwaRepository, RepositoryCall, SELLER, a_listing, a_sold_listing,
            an_open_listing, auth_context,
        },
        domain::{AlwaError, ListingStatus},
    };

    #[tokio::test]
    async fn the_seller_cancels_their_open_listing() {
        let repository = FakeAlwaRepository::new().with_listing(an_open_listing(7));
        let use_case = CancelListingUseCase::new(Arc::new(repository.clone()));

        use_case
            .execute(&auth_context(SELLER), 7)
            .await
            .expect("cancel");

        assert_eq!(
            repository
                .stored_listing(7)
                .map(|listing| *listing.status()),
            Some(ListingStatus::Cancelled)
        );
        assert_eq!(
            repository.calls(),
            vec![
                RepositoryCall::FindListingById { id: 7 },
                RepositoryCall::CancelListing { id: 7 },
            ]
        );
    }

    #[tokio::test]
    async fn someone_elses_listing_is_not_theirs_to_cancel() {
        let repository = FakeAlwaRepository::new().with_listing(an_open_listing(7));
        let use_case = CancelListingUseCase::new(Arc::new(repository.clone()));

        let result = use_case.execute(&auth_context(BUYER), 7).await;

        assert!(matches!(
            result,
            Err(AppError::Alwa(AlwaError::NotTheSeller))
        ));
        assert!(!repository.wrote(), "the listing must be left as it was");
    }

    #[tokio::test]
    async fn a_listing_that_is_not_open_is_not_cancelled() {
        let (sold, _) = a_sold_listing(7);
        let repository = FakeAlwaRepository::new()
            .with_listing(sold)
            .with_listing(a_listing(8, Utc::now() - Duration::days(5)));
        let use_case = CancelListingUseCase::new(Arc::new(repository.clone()));

        for id in [7, 8] {
            assert!(matches!(
                use_case.execute(&auth_context(SELLER), id).await,
                Err(AppError::Alwa(AlwaError::ListingNotOpen))
            ));
        }

        assert!(!repository.wrote());
    }

    #[tokio::test]
    async fn an_unknown_listing_is_not_found() {
        let use_case = CancelListingUseCase::new(Arc::new(FakeAlwaRepository::new()));

        assert!(matches!(
            use_case.execute(&auth_context(SELLER), 7).await,
            Err(AppError::GlobalAppError(GlobalAppError::NotFound))
        ));
    }
}
