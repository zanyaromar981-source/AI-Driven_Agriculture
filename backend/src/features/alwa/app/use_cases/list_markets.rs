use std::sync::Arc;

use crate::features::alwa::{
    app::{AlwaRepository, AppError},
    domain::Market,
};

pub struct ListMarketsUseCase {
    repository: Arc<dyn AlwaRepository>,
}

impl ListMarketsUseCase {
    pub fn new(repository: Arc<dyn AlwaRepository>) -> Self {
        Self { repository }
    }

    pub async fn execute(&self) -> Result<Vec<Market>, AppError> {
        let markets = self.repository.find_markets().await?;

        tracing::debug!(returned = markets.len(), "alwa markets listed");

        Ok(markets)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::features::alwa::app::testing::{FakeAlwaRepository, RepositoryCall};

    #[tokio::test]
    async fn lists_every_alwa() {
        let repository = FakeAlwaRepository::new();
        let use_case = ListMarketsUseCase::new(Arc::new(repository.clone()));

        let markets = use_case.execute().await.expect("markets");

        assert_eq!(markets.len(), 2);
        assert_eq!(markets[0].slug().as_str(), "sulaymaniyah");
        assert_eq!(repository.calls(), vec![RepositoryCall::FindMarkets]);
    }

    #[tokio::test]
    async fn a_repository_failure_surfaces() {
        let use_case = ListMarketsUseCase::new(Arc::new(FakeAlwaRepository::failing()));

        assert!(use_case.execute().await.is_err());
    }
}
