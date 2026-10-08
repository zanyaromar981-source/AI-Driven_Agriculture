use std::sync::Arc;

use crate::{
    app::AuthContext,
    features::farms::app::{AppError, FarmRepository},
};

pub struct RemoveFarmUseCase {
    repository: Arc<dyn FarmRepository>,
}

impl RemoveFarmUseCase {
    pub fn new(repository: Arc<dyn FarmRepository>) -> Self {
        Self { repository }
    }

    pub async fn execute(&self, auth_context: &AuthContext, id: i32) -> Result<(), AppError> {
        self.repository
            .delete(id, auth_context.user().phone())
            .await?;

        tracing::info!(farm_id = id, "farm removed");

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::features::farms::app::testing::{
        FakeFarmRepository, OWNER, RepositoryCall, auth_context,
    };

    #[tokio::test]
    async fn deletes_scoped_to_the_authenticated_owner() {
        let repository = FakeFarmRepository::new();
        let use_case = RemoveFarmUseCase::new(Arc::new(repository.clone()));

        let result = use_case.execute(&auth_context(), 7).await;

        assert!(result.is_ok());
        assert_eq!(
            repository.calls(),
            vec![RepositoryCall::Delete {
                id: 7,
                owner: OWNER.to_string(),
            }],
            "the owner must be part of the delete, never the id alone"
        );
    }

    #[tokio::test]
    async fn a_repository_failure_surfaces() {
        let use_case = RemoveFarmUseCase::new(Arc::new(FakeFarmRepository::failing()));

        assert!(use_case.execute(&auth_context(), 7).await.is_err());
    }
}
