use std::sync::Arc;

use crate::{
    app::{AppError as GlobalAppError, AuthContext},
    features::farms::{
        app::{AppError, FarmRepository},
        domain::Farm,
    },
};

pub struct ViewFarmUseCase {
    repository: Arc<dyn FarmRepository>,
}

impl ViewFarmUseCase {
    pub fn new(repository: Arc<dyn FarmRepository>) -> Self {
        Self { repository }
    }

    pub async fn execute(&self, auth_context: &AuthContext, id: i32) -> Result<Farm, AppError> {
        let Some(farm) = self
            .repository
            .find_by_id_and_owner(id, auth_context.user().phone())
            .await?
        else {
            tracing::info!(farm_id = id, "view refused: no such farm for this owner");

            return Err(GlobalAppError::NotFound.into());
        };

        Ok(farm)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::features::farms::app::testing::{
        FakeFarmRepository, OWNER, RepositoryCall, a_farm, auth_context,
    };

    #[tokio::test]
    async fn returns_a_farm_the_user_owns() {
        let repository = FakeFarmRepository::holding(a_farm());
        let use_case = ViewFarmUseCase::new(Arc::new(repository.clone()));

        let farm = use_case.execute(&auth_context(), 7).await.expect("farm");

        assert_eq!(*farm.id(), Some(7));
        assert_eq!(
            repository.calls(),
            vec![RepositoryCall::FindByIdAndOwner {
                id: 7,
                owner: OWNER.to_string(),
            }],
            "the owner must be part of the lookup, never the id alone"
        );
    }

    #[tokio::test]
    async fn is_not_found_when_the_user_does_not_own_it() {
        let use_case = ViewFarmUseCase::new(Arc::new(FakeFarmRepository::new()));

        let result = use_case.execute(&auth_context(), 7).await;

        assert!(matches!(
            result,
            Err(AppError::GlobalAppError(GlobalAppError::NotFound))
        ));
    }
}
