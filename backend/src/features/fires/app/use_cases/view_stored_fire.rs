use std::sync::Arc;

use crate::{
    app::AppError as GlobalAppError,
    features::fires::{
        app::{AppError, FireRepository},
        domain::Fire,
    },
};

pub struct ViewStoredFireUseCase {
    repository: Arc<dyn FireRepository>,
}

impl ViewStoredFireUseCase {
    pub fn new(repository: Arc<dyn FireRepository>) -> Self {
        Self { repository }
    }

    /// Returns one stored fire by its row id, whatever its age or status.
    pub async fn execute(&self, id: i32) -> Result<Fire, AppError> {
        self.repository
            .find_by_id(id)
            .await?
            .ok_or_else(|| GlobalAppError::NotFound.into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::features::fires::{
        app::testing::{FakeFireRepository, RepositoryCall, a_fire},
        domain::FireStatus,
    };

    #[tokio::test]
    async fn returns_the_stored_fire_however_old_it_is() {
        let repository = FakeFireRepository::holding(vec![a_fire(5, FireStatus::Out, 24 * 400)]);
        let use_case = ViewStoredFireUseCase::new(Arc::new(repository.clone()));

        let fire = use_case.execute(5).await.expect("fire");

        assert_eq!(*fire.id(), Some(5));
        assert_eq!(repository.calls(), vec![RepositoryCall::FindById { id: 5 }]);
    }

    #[tokio::test]
    async fn a_fire_that_is_not_stored_is_not_found() {
        let use_case = ViewStoredFireUseCase::new(Arc::new(FakeFireRepository::new()));

        assert!(matches!(
            use_case.execute(5).await,
            Err(AppError::GlobalAppError(GlobalAppError::NotFound))
        ));
    }

    #[tokio::test]
    async fn a_repository_failure_surfaces() {
        let use_case = ViewStoredFireUseCase::new(Arc::new(FakeFireRepository::failing()));

        assert!(matches!(
            use_case.execute(5).await,
            Err(AppError::GlobalAppError(GlobalAppError::DatabaseError(_)))
        ));
    }
}
