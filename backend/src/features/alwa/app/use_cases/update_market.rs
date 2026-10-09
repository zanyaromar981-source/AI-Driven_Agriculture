use std::sync::Arc;

use crate::{
    app::AppError as GlobalAppError,
    features::alwa::{
        app::{AlwaRepository, AppError},
        domain::{Market, MarketNames, MarketSlug},
    },
};

pub struct UpdateMarketUseCase {
    repository: Arc<dyn AlwaRepository>,
}

impl UpdateMarketUseCase {
    pub fn new(repository: Arc<dyn AlwaRepository>) -> Self {
        Self { repository }
    }

    /// A staff member renames an alwa. The slug names it and never changes.
    pub async fn execute(
        &self,
        staff_id: i32,
        slug: MarketSlug,
        names: MarketNames,
    ) -> Result<Market, AppError> {
        let Some(market) = self.repository.update_market(&slug, &names).await? else {
            tracing::info!(
                staff_id,
                market = slug.as_str(),
                "market update refused: no such alwa"
            );

            return Err(GlobalAppError::NotFound.into());
        };

        tracing::info!(staff_id, market = slug.as_str(), "alwa market updated");

        Ok(market)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::features::alwa::app::testing::{
        FakeAlwaRepository, MARKET, MARKET_ID, RepositoryCall, market_names, market_slug,
    };

    #[tokio::test]
    async fn replaces_both_names_and_keeps_the_slug_and_the_id() {
        let repository = FakeAlwaRepository::new();
        let use_case = UpdateMarketUseCase::new(Arc::new(repository.clone()));

        let market = use_case
            .execute(
                9,
                market_slug(MARKET),
                market_names("Slemani", "سلێمانی نوێ"),
            )
            .await
            .expect("market");

        assert_eq!(*market.id(), MARKET_ID);
        assert_eq!(market.slug().as_str(), MARKET);
        assert_eq!(market.name_en(), "Slemani");
        assert_eq!(repository.stored_markets()[0].name_ku(), "سلێمانی نوێ");
        assert_eq!(
            repository.calls(),
            vec![RepositoryCall::UpdateMarket {
                slug: MARKET.to_string()
            }],
            "the update decides, with no lookup before it"
        );
    }

    #[tokio::test]
    async fn an_unknown_market_is_not_found_and_not_created() {
        let repository = FakeAlwaRepository::new();
        let use_case = UpdateMarketUseCase::new(Arc::new(repository.clone()));

        let result = use_case
            .execute(9, market_slug("baghdad"), market_names("Baghdad", "بەغدا"))
            .await;

        assert!(matches!(
            result,
            Err(AppError::GlobalAppError(GlobalAppError::NotFound))
        ));
        assert_eq!(repository.stored_markets().len(), 2);
    }
}
