use std::sync::Arc;

use crate::{
    app::StaffContext,
    features::fires::app::{AppError, FireRepository},
};

pub struct RemoveFireUseCase {
    repository: Arc<dyn FireRepository>,
}

impl RemoveFireUseCase {
    pub fn new(repository: Arc<dyn FireRepository>) -> Self {
        Self { repository }
    }

    /// Removes a stored fire. A fire that is already gone is a success, so
    /// a repeated delete gets the same answer as the first.
    pub async fn execute(&self, actor: &StaffContext, id: i32) -> Result<(), AppError> {
        self.repository.delete(id).await?;

        tracing::info!(
            staff_id = *actor.staff_id(),
            fire_id = id,
            "fire removed by staff"
        );

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::features::fires::{
        app::testing::{FakeFireRepository, RepositoryCall, a_fire, actor},
        domain::FireStatus,
    };

    #[tokio::test]
    async fn removes_the_fire_in_one_call() {
        let repository = FakeFireRepository::holding(vec![
            a_fire(3, FireStatus::Active, 2),
            a_fire(4, FireStatus::Active, 1),
        ]);
        let use_case = RemoveFireUseCase::new(Arc::new(repository.clone()));

        use_case.execute(&actor(), 3).await.expect("removed");

        assert!(repository.stored(3).is_none());
        assert!(repository.stored(4).is_some(), "only that fire is removed");
        assert_eq!(repository.calls(), vec![RepositoryCall::Delete { id: 3 }]);
    }

    #[tokio::test]
    async fn removing_a_fire_that_is_already_gone_succeeds_again() {
        let use_case = RemoveFireUseCase::new(Arc::new(FakeFireRepository::new()));

        use_case.execute(&actor(), 3).await.expect("first");
        use_case.execute(&actor(), 3).await.expect("repeat");
    }

    #[tokio::test]
    async fn a_repository_failure_surfaces() {
        let use_case = RemoveFireUseCase::new(Arc::new(FakeFireRepository::failing()));

        assert!(use_case.execute(&actor(), 3).await.is_err());
    }
}
