use std::sync::Arc;

use crate::{
    app::StaffContext,
    features::workers::app::{AppError, WorkerRepository},
};

pub struct DeleteWorkerUseCase {
    repository: Arc<dyn WorkerRepository>,
}

impl DeleteWorkerUseCase {
    pub fn new(repository: Arc<dyn WorkerRepository>) -> Self {
        Self { repository }
    }

    /// Removes any card. Removing one that is not there succeeds: a repeat
    /// of a delete is the same delete.
    pub async fn execute(&self, actor: &StaffContext, id: i32) -> Result<(), AppError> {
        let removed = self.repository.delete(id).await?;

        if removed {
            tracing::info!(
                staff_id = *actor.staff_id(),
                worker_id = id,
                "worker card removed by staff"
            );
        } else {
            tracing::info!(
                staff_id = *actor.staff_id(),
                worker_id = id,
                "worker card already gone: nothing to remove"
            );
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::features::workers::app::testing::{
        Call, Fakes, OTHER_PHONE, PHONE, a_worker, staff_context,
    };

    #[tokio::test]
    async fn removes_that_card_only() {
        let fakes = Fakes::new()
            .with_stored(a_worker(1, PHONE))
            .with_stored(a_worker(2, OTHER_PHONE));

        DeleteWorkerUseCase::new(Arc::new(fakes.clone()))
            .execute(&staff_context(), 1)
            .await
            .expect("removed");

        assert_eq!(fakes.stored(), vec![a_worker(2, OTHER_PHONE)]);
        assert_eq!(fakes.calls(), vec![Call::Delete { id: 1 }]);
    }

    #[tokio::test]
    async fn removing_a_card_that_is_gone_succeeds() {
        let use_case = DeleteWorkerUseCase::new(Arc::new(Fakes::new()));

        assert!(use_case.execute(&staff_context(), 1).await.is_ok());
        assert!(use_case.execute(&staff_context(), 1).await.is_ok());
    }

    #[tokio::test]
    async fn a_repository_failure_surfaces() {
        let result = DeleteWorkerUseCase::new(Arc::new(Fakes::new().failing()))
            .execute(&staff_context(), 1)
            .await;

        assert!(result.is_err());
    }
}
