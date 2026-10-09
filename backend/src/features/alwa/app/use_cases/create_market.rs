use std::sync::Arc;

use crate::features::alwa::{
    app::{AlwaRepository, AppError},
    domain::{AlwaError, Market, MarketNames, MarketSlug},
};

pub struct CreateMarketInput {
    pub slug: MarketSlug,
    pub names: MarketNames,
}

pub struct CreateMarketUseCase {
    repository: Arc<dyn AlwaRepository>,
}

impl CreateMarketUseCase {
    pub fn new(repository: Arc<dyn AlwaRepository>) -> Self {
        Self { repository }
    }

    /// A staff member adds an alwa. A slug that is taken is refused, and the
    /// market that has it is left as it was.
    pub async fn execute(
        &self,
        staff_id: i32,
        input: CreateMarketInput,
    ) -> Result<Market, AppError> {
        let Some(market) = self
            .repository
            .create_market(&input.slug, &input.names)
            .await?
        else {
            tracing::info!(
                staff_id,
                market = input.slug.as_str(),
                "market refused: the slug is taken"
            );

            return Err(AlwaError::AlreadyExists("market").into());
        };

        tracing::info!(
            staff_id,
            market = market.slug().as_str(),
            "alwa market created"
        );

        Ok(market)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::features::alwa::{
        app::testing::{FakeAlwaRepository, MARKET, RepositoryCall, market_names, market_slug},
        domain::MarketName,
    };

    fn input(slug: &str) -> CreateMarketInput {
        CreateMarketInput {
            slug: market_slug(slug),
            names: market_names("Halabja", "هەڵەبجە"),
        }
    }

    #[tokio::test]
    async fn adds_a_market_under_a_new_slug() {
        let repository = FakeAlwaRepository::new();
        let use_case = CreateMarketUseCase::new(Arc::new(repository.clone()));

        let market = use_case.execute(9, input("halabja")).await.expect("market");

        assert_eq!(market.slug().as_str(), "halabja");
        assert_eq!(market.name_ku(), "هەڵەبجە");
        assert_eq!(repository.stored_markets().len(), 3);
        assert_eq!(
            repository.calls(),
            vec![RepositoryCall::CreateMarket {
                slug: "halabja".to_string()
            }],
            "the insert decides, with no lookup before it"
        );
    }

    #[tokio::test]
    async fn a_taken_slug_is_refused_and_the_market_keeps_its_names() {
        let repository = FakeAlwaRepository::new();
        let use_case = CreateMarketUseCase::new(Arc::new(repository.clone()));

        let result = use_case.execute(9, input(MARKET)).await;

        assert!(matches!(
            result,
            Err(AppError::Alwa(AlwaError::AlreadyExists("market")))
        ));
        assert_eq!(
            repository.stored_markets()[0].name_en(),
            MarketName::new("Sulaymaniyah".to_string())
                .expect("name")
                .as_str()
        );
    }

    #[tokio::test]
    async fn a_repository_failure_surfaces() {
        let use_case = CreateMarketUseCase::new(Arc::new(FakeAlwaRepository::failing()));

        assert!(use_case.execute(9, input("halabja")).await.is_err());
    }
}
