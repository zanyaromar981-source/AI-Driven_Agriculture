use std::sync::Arc;

use chrono::{DateTime, Utc};

use crate::{
    app::AuthContext,
    features::farms::{
        app::{AppError, FarmRepository},
        domain::{Farm, FarmName, GridCell, Outline, PaintedCell},
    },
};

pub struct RegisterFarmInput {
    pub name: FarmName,
    pub outline: Outline,
    pub painted: Vec<PaintedCell>,
    pub created_offline_at: Option<DateTime<Utc>>,
}

pub struct RegisterFarmUseCase {
    repository: Arc<dyn FarmRepository>,
    max_farms_per_user: u64,
    max_cells_per_farm: usize,
}

impl RegisterFarmUseCase {
    pub fn new(
        repository: Arc<dyn FarmRepository>,
        max_farms_per_user: u64,
        max_cells_per_farm: usize,
    ) -> Self {
        Self {
            repository,
            max_farms_per_user,
            max_cells_per_farm,
        }
    }

    /// Returns the registered farm and the painted cells that were dropped
    /// because they fall outside its outline.
    pub async fn execute(
        &self,
        auth_context: &AuthContext,
        input: RegisterFarmInput,
    ) -> Result<(Farm, Vec<GridCell>), AppError> {
        let owner = auth_context.user().phone();

        let owned = self.repository.count_by_owner(owner).await?;

        if owned >= self.max_farms_per_user {
            tracing::info!(
                owned,
                max = self.max_farms_per_user,
                "registration refused: farm quota reached"
            );

            return Err(AppError::MaxFarmsPerUserReached(self.max_farms_per_user));
        }

        let (farm, dropped_cells) = Farm::new(
            input.name,
            owner.clone(),
            input.outline,
            input.painted,
            input.created_offline_at,
            self.max_cells_per_farm,
        )?;

        let registered = self.repository.create(&farm).await?;

        tracing::info!(
            farm_id = registered.id().unwrap_or_default(),
            cells = registered.cells().len(),
            dropped_cells = dropped_cells.len(),
            owned_after = owned + 1,
            "farm registered"
        );

        Ok((registered, dropped_cells))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::features::farms::{
        app::testing::{
            FakeFarmRepository, MAX_CELLS, OWNER, RepositoryCall, a_cell_inside, a_cell_outside,
            an_outline, auth_context,
        },
        domain::{Crop, FarmError},
    };

    const MAX: u64 = 3;

    fn input(painted: Vec<PaintedCell>) -> RegisterFarmInput {
        RegisterFarmInput {
            name: FarmName::new("Upper field".to_string()).expect("name"),
            outline: an_outline(),
            painted,
            created_offline_at: None,
        }
    }

    async fn register(
        repository: FakeFarmRepository,
        max_cells: usize,
        painted: Vec<PaintedCell>,
    ) -> (Result<(Farm, Vec<GridCell>), AppError>, FakeFarmRepository) {
        let use_case = RegisterFarmUseCase::new(Arc::new(repository.clone()), MAX, max_cells);
        let result = use_case.execute(&auth_context(), input(painted)).await;

        (result, repository)
    }

    #[tokio::test]
    async fn registers_a_farm_when_under_the_quota() {
        let (result, repository) =
            register(FakeFarmRepository::owning(MAX - 1), MAX_CELLS, vec![]).await;

        assert!(result.is_ok());
        assert!(repository.calls().contains(&RepositoryCall::Create));
    }

    #[tokio::test]
    async fn the_farm_belongs_to_the_authenticated_phone() {
        let (result, _) = register(FakeFarmRepository::new(), MAX_CELLS, vec![]).await;

        let (farm, _) = result.expect("farm");

        assert_eq!(farm.owner().as_str(), OWNER);
    }

    #[tokio::test]
    async fn rejects_once_the_quota_is_reached() {
        let (result, repository) =
            register(FakeFarmRepository::owning(MAX), MAX_CELLS, vec![]).await;

        assert!(matches!(result, Err(AppError::MaxFarmsPerUserReached(MAX))));
        assert!(
            !repository.calls().contains(&RepositoryCall::Create),
            "a rejected registration must not write"
        );
    }

    #[tokio::test]
    async fn the_quota_is_a_ceiling_not_a_limit_to_exceed() {
        let (over, _) = register(FakeFarmRepository::owning(MAX + 1), MAX_CELLS, vec![]).await;

        assert!(matches!(over, Err(AppError::MaxFarmsPerUserReached(MAX))));
    }

    #[tokio::test]
    async fn the_quota_is_counted_for_the_authenticated_owner() {
        let (_, repository) = register(FakeFarmRepository::owning(MAX), MAX_CELLS, vec![]).await;

        assert_eq!(
            repository.calls(),
            vec![RepositoryCall::CountByOwner {
                owner: OWNER.to_string()
            }],
            "the quota rejection must short-circuit before anything else"
        );
    }

    #[tokio::test]
    async fn painted_cells_outside_the_outline_are_reported_as_dropped() {
        let (result, _) = register(
            FakeFarmRepository::new(),
            MAX_CELLS,
            vec![
                PaintedCell::new(a_cell_inside(), Crop::Wheat),
                PaintedCell::new(a_cell_outside(), Crop::Wheat),
            ],
        )
        .await;

        let (farm, dropped) = result.expect("farm");

        assert_eq!(dropped, vec![a_cell_outside()]);
        assert_eq!(farm.crop_areas().len(), 1);
    }

    #[tokio::test]
    async fn a_farm_over_the_cell_limit_is_not_written() {
        let (result, repository) = register(FakeFarmRepository::new(), 10, vec![]).await;

        assert!(matches!(
            result,
            Err(AppError::Farm(FarmError::TooManyCells(10)))
        ));
        assert!(!repository.calls().contains(&RepositoryCall::Create));
    }

    #[tokio::test]
    async fn a_repository_failure_surfaces_instead_of_registering() {
        let (result, repository) = register(FakeFarmRepository::failing(), MAX_CELLS, vec![]).await;

        assert!(result.is_err());
        assert!(!repository.calls().contains(&RepositoryCall::Create));
    }
}
