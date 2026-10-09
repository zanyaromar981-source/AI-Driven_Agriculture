use std::sync::Arc;

use crate::{
    app::AppError as GlobalAppError,
    features::farms::{
        app::{AppError, FarmRepository},
        domain::Farm,
    },
};

/// The dashboard's view of one farm, whoever owns it. The farmer app views
/// through `ViewFarmUseCase`, scoped to its own phone.
pub struct ViewAnyFarmUseCase {
    repository: Arc<dyn FarmRepository>,
}

impl ViewAnyFarmUseCase {
    pub fn new(repository: Arc<dyn FarmRepository>) -> Self {
        Self { repository }
    }

    pub async fn execute(&self, id: i32) -> Result<Farm, AppError> {
        self.repository
            .find_by_id(id)
            .await?
            .ok_or_else(|| GlobalAppError::NotFound.into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::features::farms::app::testing::{FakeFarmRepository, RepositoryCall, a_farm};

    #[tokio::test]
    async fn returns_the_farm_with_its_owner() {
        let repository = FakeFarmRepository::holding(a_farm());
        let use_case = ViewAnyFarmUseCase::new(Arc::new(repository.clone()));

        let farm = use_case.execute(7).await.expect("farm");

        assert_eq!(*farm.id(), Some(7));
        assert_eq!(repository.calls(), vec![RepositoryCall::FindById { id: 7 }]);
    }

    #[tokio::test]
    async fn is_not_found_when_there_is_no_such_farm() {
        let use_case = ViewAnyFarmUseCase::new(Arc::new(FakeFarmRepository::new()));

        assert!(matches!(
            use_case.execute(7).await,
            Err(AppError::GlobalAppError(GlobalAppError::NotFound))
        ));
    }
}
