use std::sync::Arc;

use crate::features::alwa::{
    app::{AlwaRepository, AppError},
    domain::MarketSlug,
};

pub struct DeleteMarketUseCase {
    repository: Arc<dyn AlwaRepository>,
}

impl DeleteMarketUseCase {
    pub fn new(repository: Arc<dyn AlwaRepository>) -> Self {
        Self { repository }
    }

    /// A staff member removes an alwa that has no prices and no listings.
    /// Removing one that is already gone succeeds: gone is what was asked.
    pub async fn execute(&self, staff_id: i32, slug: MarketSlug) -> Result<(), AppError> {
        let removed = self
            .repository
            .delete_market(&slug)
            .await
            .inspect_err(|error| {
                tracing::info!(staff_id, market = slug.as_str(), %error, "market delete refused");
            })?;

        tracing::info!(
            staff_id,
            market = slug.as_str(),
            removed,
            "alwa market deleted"
        );

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use chrono::Utc;

    use super::*;
    use crate::features::alwa::{
        app::testing::{
            FakeAlwaRepository, MARKET, MARKET_ID, RepositoryCall, a_price, an_open_listing,
            market_slug,
        },
        domain::{AlwaError, Crop},
    };

    #[tokio::test]
    async fn removes_a_market_nothing_points_at() {
        let repository = FakeAlwaRepository::new();
        let use_case = DeleteMarketUseCase::new(Arc::new(repository.clone()));

        use_case
            .execute(9, market_slug("erbil"))
            .await
            .expect("delete");

        assert_eq!(repository.stored_markets().len(), 1);
        assert_eq!(
            repository.calls(),
            vec![RepositoryCall::DeleteMarket {
                slug: "erbil".to_string()
            }],
            "the delete decides, with no count read before it"
        );
    }

    #[tokio::test]
    async fn deleting_again_or_deleting_an_unknown_market_succeeds() {
        let repository = FakeAlwaRepository::new();
        let use_case = DeleteMarketUseCase::new(Arc::new(repository.clone()));

        use_case
            .execute(9, market_slug("erbil"))
            .await
            .expect("first");

        assert!(use_case.execute(9, market_slug("erbil")).await.is_ok());
        assert!(use_case.execute(9, market_slug("baghdad")).await.is_ok());
    }

    #[tokio::test]
    async fn a_market_with_a_price_or_a_listing_stays() {
        let with_price = FakeAlwaRepository::new().with_price(a_price(
            MARKET_ID,
            Crop::Tomato,
            Utc::now().date_naive(),
            900,
            false,
        ));
        let with_listing = FakeAlwaRepository::new().with_listing(an_open_listing(7));

        for repository in [with_price, with_listing] {
            let use_case = DeleteMarketUseCase::new(Arc::new(repository.clone()));

            assert!(matches!(
                use_case.execute(9, market_slug(MARKET)).await,
                Err(AppError::Alwa(AlwaError::MarketInUse))
            ));
            assert_eq!(repository.stored_markets().len(), 2);
        }
    }
}
