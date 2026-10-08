use std::sync::Arc;

use crate::{
    app::{AppError as GlobalAppError, AuthContext},
    features::farms::{
        app::{AppError, FarmRepository},
        domain::{Farm, GridCell, PaintedCell},
    },
};

pub struct RepaintFarmCellsInput {
    pub painted: Vec<PaintedCell>,
}

pub struct RepaintFarmCellsUseCase {
    repository: Arc<dyn FarmRepository>,
}

impl RepaintFarmCellsUseCase {
    pub fn new(repository: Arc<dyn FarmRepository>) -> Self {
        Self { repository }
    }

    /// Returns the repainted farm and the cells that were dropped because
    /// they are not part of it.
    pub async fn execute(
        &self,
        auth_context: &AuthContext,
        id: i32,
        input: RepaintFarmCellsInput,
    ) -> Result<(Farm, Vec<GridCell>), AppError> {
        let Some(mut farm) = self
            .repository
            .find_by_id_and_owner(id, auth_context.user().phone())
            .await?
        else {
            tracing::info!(farm_id = id, "repaint refused: no such farm for this owner");

            return Err(GlobalAppError::NotFound.into());
        };

        let dropped_cells = farm.repaint(input.painted);

        let updated = self.repository.update(&farm).await?;

        tracing::info!(
            farm_id = id,
            dropped_cells = dropped_cells.len(),
            "farm cells repainted"
        );

        Ok((updated, dropped_cells))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::features::farms::{
        app::testing::{
            FakeFarmRepository, OWNER, RepositoryCall, a_cell_inside, a_cell_outside, a_farm,
            auth_context,
        },
        domain::Crop,
    };

    fn input(position: GridCell, crop: Crop) -> RepaintFarmCellsInput {
        RepaintFarmCellsInput {
            painted: vec![PaintedCell::new(position, crop)],
        }
    }

    #[tokio::test]
    async fn repaints_a_farm_the_user_owns() {
        let repository = FakeFarmRepository::holding(a_farm());
        let use_case = RepaintFarmCellsUseCase::new(Arc::new(repository.clone()));

        let (farm, dropped) = use_case
            .execute(&auth_context(), 7, input(a_cell_inside(), Crop::Barley))
            .await
            .expect("repaint");

        assert_eq!(farm.crop_areas()[0].crop(), Crop::Barley);
        assert!(dropped.is_empty());
        assert!(repository.calls().contains(&RepositoryCall::Update));
    }

    #[tokio::test]
    async fn is_not_found_when_the_user_does_not_own_it() {
        let repository = FakeFarmRepository::new();
        let use_case = RepaintFarmCellsUseCase::new(Arc::new(repository.clone()));

        let result = use_case
            .execute(&auth_context(), 7, input(a_cell_inside(), Crop::Barley))
            .await;

        assert!(matches!(
            result,
            Err(AppError::GlobalAppError(GlobalAppError::NotFound))
        ));
        assert!(
            !repository.calls().contains(&RepositoryCall::Update),
            "a missing farm must not be written"
        );
    }

    #[tokio::test]
    async fn looks_the_farm_up_scoped_to_the_owner() {
        let repository = FakeFarmRepository::holding(a_farm());
        let use_case = RepaintFarmCellsUseCase::new(Arc::new(repository.clone()));

        let _ = use_case
            .execute(&auth_context(), 7, input(a_cell_inside(), Crop::Barley))
            .await;

        assert!(
            repository
                .calls()
                .contains(&RepositoryCall::FindByIdAndOwner {
                    id: 7,
                    owner: OWNER.to_string(),
                })
        );
    }

    #[tokio::test]
    async fn a_cell_outside_the_farm_is_reported_as_dropped() {
        let repository = FakeFarmRepository::holding(a_farm());
        let use_case = RepaintFarmCellsUseCase::new(Arc::new(repository.clone()));

        let (farm, dropped) = use_case
            .execute(&auth_context(), 7, input(a_cell_outside(), Crop::Barley))
            .await
            .expect("repaint");

        assert_eq!(dropped, vec![a_cell_outside()]);
        assert_eq!(
            farm.crop_areas()[0].crop(),
            Crop::Wheat,
            "the farm keeps the crops it had"
        );
    }
}
