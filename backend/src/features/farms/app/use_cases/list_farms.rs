use std::sync::Arc;

use crate::{
    app::AuthContext,
    features::farms::{
        app::{AppError, FarmRepository},
        domain::FarmSummary,
    },
};

pub struct ListFarmsUseCase {
    repository: Arc<dyn FarmRepository>,
}

impl ListFarmsUseCase {
    pub fn new(repository: Arc<dyn FarmRepository>) -> Self {
        Self { repository }
    }

    pub async fn execute(&self, auth_context: &AuthContext) -> Result<Vec<FarmSummary>, AppError> {
        let farms = self
            .repository
            .find_all_by_owner(auth_context.user().phone())
            .await?;

        tracing::debug!(returned = farms.len(), "farms listed");

        Ok(farms)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::features::farms::app::testing::{
        FakeFarmRepository, OWNER, RepositoryCall, a_farm, auth_context,
    };

    #[tokio::test]
    async fn lists_only_the_authenticated_owners_farms() {
        let repository = FakeFarmRepository::holding(a_farm());
        let use_case = ListFarmsUseCase::new(Arc::new(repository.clone()));

        let rows = use_case.execute(&auth_context()).await.expect("listing");

        assert_eq!(rows.len(), 1);
        assert_eq!(
            repository.calls(),
            vec![RepositoryCall::FindAllByOwner {
                owner: OWNER.to_string(),
            }]
        );
    }

    #[tokio::test]
    async fn a_repository_failure_surfaces() {
        let use_case = ListFarmsUseCase::new(Arc::new(FakeFarmRepository::failing()));

        assert!(use_case.execute(&auth_context()).await.is_err());
    }
}
