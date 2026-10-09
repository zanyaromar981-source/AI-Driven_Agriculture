use std::sync::Arc;

use crate::{
    app::StaffContext,
    features::farms::app::{AppError, FarmRepository},
};

/// Staff removing a farm, whoever owns it. The farmer app removes through
/// `RemoveFarmUseCase`, scoped to its own phone.
pub struct RemoveAnyFarmUseCase {
    repository: Arc<dyn FarmRepository>,
}

impl RemoveAnyFarmUseCase {
    pub fn new(repository: Arc<dyn FarmRepository>) -> Self {
        Self { repository }
    }

    /// Removing a farm that is not there succeeds: it being gone is what
    /// was asked for.
    pub async fn execute(&self, actor: &StaffContext, id: i32) -> Result<(), AppError> {
        let removed = self.repository.delete_by_id(id).await?;

        tracing::info!(
            staff_id = *actor.staff_id(),
            farm_id = id,
            removed,
            "farm removed by staff"
        );

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::features::farms::app::testing::{FakeFarmRepository, RepositoryCall, staff_context};

    #[tokio::test]
    async fn deletes_the_farm_by_its_id() {
        let repository = FakeFarmRepository::new();
        let use_case = RemoveAnyFarmUseCase::new(Arc::new(repository.clone()));

        assert!(use_case.execute(&staff_context(), 7).await.is_ok());
        assert_eq!(
            repository.calls(),
            vec![RepositoryCall::DeleteById { id: 7 }]
        );
    }

    #[tokio::test]
    async fn removing_a_farm_that_is_already_gone_succeeds() {
        let use_case =
            RemoveAnyFarmUseCase::new(Arc::new(FakeFarmRepository::holding_nothing_to_delete()));

        assert!(
            use_case.execute(&staff_context(), 7).await.is_ok(),
            "a repeated delete must not turn into an error"
        );
    }

    #[tokio::test]
    async fn a_repository_failure_surfaces() {
        let use_case = RemoveAnyFarmUseCase::new(Arc::new(FakeFarmRepository::failing()));

        assert!(use_case.execute(&staff_context(), 7).await.is_err());
    }
}
