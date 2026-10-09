use std::sync::Arc;

use crate::{
    app::StaffContext,
    features::messages::app::{AppError, MessageRepository},
};

pub struct DeleteMessageUseCase {
    repository: Arc<dyn MessageRepository>,
}

impl DeleteMessageUseCase {
    pub fn new(repository: Arc<dyn MessageRepository>) -> Self {
        Self { repository }
    }

    /// Removes a message and its photos. Removing one that is not there
    /// succeeds: a repeat of a delete is the same delete.
    pub async fn execute(&self, actor: &StaffContext, id: i32) -> Result<(), AppError> {
        let removed = self.repository.delete(id).await?;

        if removed {
            tracing::info!(
                staff_id = *actor.staff_id(),
                message_id = id,
                "message removed by staff"
            );
        } else {
            tracing::info!(
                staff_id = *actor.staff_id(),
                message_id = id,
                "message already gone: nothing to remove"
            );
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::features::messages::app::testing::{Call, Fakes, a_message, staff_context};

    #[tokio::test]
    async fn removes_the_message() {
        let fakes = Fakes::new().with_stored(a_message(1));

        DeleteMessageUseCase::new(Arc::new(fakes.clone()))
            .execute(&staff_context(), 1)
            .await
            .expect("removed");

        assert!(fakes.stored().is_empty());
        assert_eq!(fakes.calls(), vec![Call::Delete { id: 1 }]);
    }

    #[tokio::test]
    async fn removing_a_message_that_is_gone_succeeds() {
        let fakes = Fakes::new();
        let use_case = DeleteMessageUseCase::new(Arc::new(fakes.clone()));

        assert!(use_case.execute(&staff_context(), 1).await.is_ok());
        assert!(use_case.execute(&staff_context(), 1).await.is_ok());
    }

    #[tokio::test]
    async fn a_repository_failure_surfaces() {
        let result = DeleteMessageUseCase::new(Arc::new(Fakes::new().failing()))
            .execute(&staff_context(), 1)
            .await;

        assert!(result.is_err());
    }
}
