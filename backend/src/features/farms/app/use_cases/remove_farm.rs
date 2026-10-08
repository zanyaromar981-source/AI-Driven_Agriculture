use std::sync::Arc;

use crate::{
    app::{AppError as GlobalAppError, AuthContext},
    features::farms::app::{AppError, FarmRepository},
};

pub struct RemoveFarmUseCase {
    repository: Arc<dyn FarmRepository>,
}

impl RemoveFarmUseCase {
    pub fn new(repository: Arc<dyn FarmRepository>) -> Self {
        Self { repository }
    }

    /// Removing a farm that is not there succeeds: the app repeats a delete
    /// whose answer was lost, and the farm being gone is what it asked for.
    /// It also means the answer does not say whether another farmer's farm
    /// exists.
    pub async fn execute(&self, auth_context: &AuthContext, id: i32) -> Result<(), AppError> {
        match self
            .repository
            .delete(id, auth_context.user().phone())
            .await
        {
            Ok(()) => tracing::info!(farm_id = id, "farm removed"),
            Err(AppError::GlobalAppError(GlobalAppError::NotFound)) => {
                tracing::info!(farm_id = id, "farm already gone: nothing to remove");
            }
            Err(error) => return Err(error),
        }

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
    async fn removing_a_farm_that_is_already_gone_succeeds() {
        let use_case =
            RemoveFarmUseCase::new(Arc::new(FakeFarmRepository::holding_nothing_to_delete()));

        assert!(
            use_case.execute(&auth_context(), 7).await.is_ok(),
            "a repeated delete must not turn into an error"
        );
    }

    #[tokio::test]
    async fn a_repository_failure_surfaces() {
        let use_case = RemoveFarmUseCase::new(Arc::new(FakeFarmRepository::failing()));

        assert!(use_case.execute(&auth_context(), 7).await.is_err());
    }
}
