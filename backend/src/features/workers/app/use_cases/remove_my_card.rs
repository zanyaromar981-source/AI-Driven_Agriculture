use std::sync::Arc;

use crate::{
    app::AuthContext,
    features::workers::app::{AppError, WorkerRepository},
};

pub struct RemoveMyCardUseCase {
    repository: Arc<dyn WorkerRepository>,
}

impl RemoveMyCardUseCase {
    pub fn new(repository: Arc<dyn WorkerRepository>) -> Self {
        Self { repository }
    }

    /// Takes the caller's card down for good. Removing a card that is not
    /// there succeeds: a repeat of a delete is the same delete.
    pub async fn execute(&self, auth_context: &AuthContext) -> Result<(), AppError> {
        let removed = self
            .repository
            .delete_by_phone(auth_context.user().phone())
            .await?;

        tracing::info!(removed, "worker card removed by its owner");

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::features::workers::app::testing::{
        Call, Fakes, OTHER_PHONE, PHONE, a_worker, auth_context,
    };

    #[tokio::test]
    async fn removes_the_callers_card_and_nobody_elses() {
        let fakes = Fakes::new()
            .with_stored(a_worker(1, OTHER_PHONE))
            .with_stored(a_worker(2, PHONE));

        RemoveMyCardUseCase::new(Arc::new(fakes.clone()))
            .execute(&auth_context())
            .await
            .expect("removed");

        assert_eq!(fakes.stored(), vec![a_worker(1, OTHER_PHONE)]);
        assert_eq!(
            fakes.calls(),
            vec![Call::DeleteByPhone {
                phone: PHONE.to_string()
            }]
        );
    }

    #[tokio::test]
    async fn removing_a_card_that_is_gone_succeeds() {
        let use_case = RemoveMyCardUseCase::new(Arc::new(Fakes::new()));

        assert!(use_case.execute(&auth_context()).await.is_ok());
        assert!(use_case.execute(&auth_context()).await.is_ok());
    }

    #[tokio::test]
    async fn a_repository_failure_surfaces() {
        let result = RemoveMyCardUseCase::new(Arc::new(Fakes::new().failing()))
            .execute(&auth_context())
            .await;

        assert!(result.is_err());
    }
}
