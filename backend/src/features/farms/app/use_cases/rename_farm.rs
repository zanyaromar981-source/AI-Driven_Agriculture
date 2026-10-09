use std::sync::Arc;

use chrono::Utc;

use crate::{
    app::{AppError as GlobalAppError, StaffContext},
    features::farms::{
        app::{AppError, FarmRepository},
        domain::{Farm, FarmName},
    },
};

pub struct RenameFarmInput {
    pub name: FarmName,
}

/// Staff renaming a farm. The outline and the cells are the farmer's and
/// are not touched: one statement writes the name and nothing else.
pub struct RenameFarmUseCase {
    repository: Arc<dyn FarmRepository>,
}

impl RenameFarmUseCase {
    pub fn new(repository: Arc<dyn FarmRepository>) -> Self {
        Self { repository }
    }

    pub async fn execute(
        &self,
        actor: &StaffContext,
        id: i32,
        input: RenameFarmInput,
    ) -> Result<Farm, AppError> {
        let Some(farm) = self.repository.rename(id, &input.name, Utc::now()).await? else {
            tracing::info!(
                staff_id = *actor.staff_id(),
                farm_id = id,
                "farm rename by staff refused: no such farm"
            );

            return Err(GlobalAppError::NotFound.into());
        };

        tracing::info!(
            staff_id = *actor.staff_id(),
            farm_id = id,
            "farm renamed by staff"
        );

        Ok(farm)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::features::farms::app::testing::{
        FakeFarmRepository, RepositoryCall, a_farm, staff_context,
    };

    fn input() -> RenameFarmInput {
        RenameFarmInput {
            name: FarmName::new("Lower field".to_string()).expect("name"),
        }
    }

    #[tokio::test]
    async fn renames_the_farm_and_leaves_its_cells_alone() {
        let before = a_farm();
        let repository = FakeFarmRepository::holding(before.clone());
        let use_case = RenameFarmUseCase::new(Arc::new(repository.clone()));

        let farm = use_case
            .execute(&staff_context(), 7, input())
            .await
            .expect("farm");

        assert_eq!(farm.name().as_str(), "Lower field");
        assert_eq!(farm.cells(), before.cells());
        assert_eq!(
            repository.calls(),
            vec![RepositoryCall::Rename {
                id: 7,
                name: "Lower field".to_string(),
            }],
            "the rename is one write, with no lookup before it and no whole-farm update"
        );
    }

    #[tokio::test]
    async fn is_not_found_when_there_is_no_such_farm() {
        let use_case = RenameFarmUseCase::new(Arc::new(FakeFarmRepository::new()));

        assert!(matches!(
            use_case.execute(&staff_context(), 7, input()).await,
            Err(AppError::GlobalAppError(GlobalAppError::NotFound))
        ));
    }
}
