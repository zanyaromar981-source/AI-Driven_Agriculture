use std::sync::Arc;

use crate::{
    app::{AppError as GlobalAppError, AuthContext},
    features::farms::{
        app::{AppError, FarmRepository},
        domain::{Farm, FarmName, GridCell, Outline, PaintedCell},
    },
};

pub struct EditFarmInput {
    pub name: FarmName,
    pub outline: Outline,
    pub painted: Vec<PaintedCell>,
}

/// The farmer changing a farm in the app: a new border, new crops and a new
/// name in one request that carries the whole farm. The farm keeps its id.
pub struct EditFarmUseCase {
    repository: Arc<dyn FarmRepository>,
    max_cells_per_farm: usize,
}

impl EditFarmUseCase {
    pub fn new(repository: Arc<dyn FarmRepository>, max_cells_per_farm: usize) -> Self {
        Self {
            repository,
            max_cells_per_farm,
        }
    }

    /// Returns the edited farm and the painted cells that were dropped
    /// because the new outline does not touch them.
    pub async fn execute(
        &self,
        auth_context: &AuthContext,
        id: i32,
        input: EditFarmInput,
    ) -> Result<(Farm, Vec<GridCell>), AppError> {
        let Some(mut farm) = self
            .repository
            .find_by_id_and_owner(id, auth_context.user().phone())
            .await?
        else {
            tracing::info!(farm_id = id, "edit refused: no such farm for this owner");

            return Err(GlobalAppError::NotFound.into());
        };

        let redraw = farm.redraw(
            input.name,
            input.outline,
            input.painted,
            self.max_cells_per_farm,
        )?;

        // The same edit arriving again finds the farm already as it asks,
        // and gets that farm back unchanged. Skipping the write cannot lose
        // anything: had another edit landed since the lookup, this one would
        // simply count as the earlier of the two.
        if !redraw.changed {
            tracing::info!(farm_id = id, "edit repeated: the farm is already as sent");

            return Ok((farm, redraw.dropped_cells));
        }

        // The lookup above only chose the answer for a missing farm. The
        // write decides for itself, under a lock on the farm's row, whether
        // the farm is still there and still this owner's.
        let edited = self.repository.replace(&farm).await?;

        tracing::info!(
            farm_id = id,
            cells = edited.cells().len(),
            dropped_cells = redraw.dropped_cells.len(),
            "farm edited"
        );

        Ok((edited, redraw.dropped_cells))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::features::farms::{
        app::testing::{
            FakeFarmRepository, MAX_CELLS, OWNER, RepositoryCall, a_cell_inside, a_farm,
            an_outline, another_outline, auth_context,
        },
        domain::{Crop, FarmError},
    };

    fn name(text: &str) -> FarmName {
        FarmName::new(text.to_string()).expect("name")
    }

    /// A new name, a smaller outline and one cell of barley.
    fn an_edit() -> EditFarmInput {
        let inside_the_new = another_outline().cells(MAX_CELLS).expect("cells")[0].position();

        EditFarmInput {
            name: name("Lower field"),
            outline: another_outline(),
            painted: vec![PaintedCell::new(inside_the_new, Crop::Barley)],
        }
    }

    fn use_case(repository: &FakeFarmRepository) -> EditFarmUseCase {
        EditFarmUseCase::new(Arc::new(repository.clone()), MAX_CELLS)
    }

    #[tokio::test]
    async fn replaces_the_name_the_outline_and_the_cells_and_keeps_the_id() {
        let repository = FakeFarmRepository::holding(a_farm());

        let (farm, dropped) = use_case(&repository)
            .execute(&auth_context(), 7, an_edit())
            .await
            .expect("edit");

        assert_eq!(*farm.id(), Some(7));
        assert_eq!(farm.name().as_str(), "Lower field");
        assert_eq!(farm.outline(), &another_outline());
        assert_eq!(
            farm.cells().len(),
            another_outline().cells(MAX_CELLS).expect("cells").len()
        );
        assert_eq!(farm.crop_areas().len(), 1);
        assert_eq!(farm.crop_areas()[0].crop(), Crop::Barley);
        assert!(dropped.is_empty());
    }

    #[tokio::test]
    async fn looks_the_farm_up_and_writes_it_scoped_to_the_owner_in_that_order() {
        let repository = FakeFarmRepository::holding(a_farm());

        let _ = use_case(&repository)
            .execute(&auth_context(), 7, an_edit())
            .await;

        assert_eq!(
            repository.calls(),
            vec![
                RepositoryCall::FindByIdAndOwner {
                    id: 7,
                    owner: OWNER.to_string(),
                },
                RepositoryCall::Replace {
                    id: 7,
                    owner: OWNER.to_string(),
                },
            ]
        );
    }

    #[tokio::test]
    async fn is_not_found_when_the_user_does_not_own_it() {
        let repository = FakeFarmRepository::new();

        let result = use_case(&repository)
            .execute(&auth_context(), 7, an_edit())
            .await;

        assert!(matches!(
            result,
            Err(AppError::GlobalAppError(GlobalAppError::NotFound))
        ));
        assert!(
            !repository
                .calls()
                .iter()
                .any(|call| matches!(call, RepositoryCall::Replace { .. })),
            "a missing farm must not be written"
        );
    }

    #[tokio::test]
    async fn is_not_found_when_the_farm_is_deleted_between_the_lookup_and_the_write() {
        let repository = FakeFarmRepository::holding_one_that_vanishes(a_farm());

        let result = use_case(&repository)
            .execute(&auth_context(), 7, an_edit())
            .await;

        assert!(
            matches!(
                result,
                Err(AppError::GlobalAppError(GlobalAppError::NotFound))
            ),
            "a farm deleted meanwhile is gone, not a server fault"
        );
    }

    #[tokio::test]
    async fn the_same_edit_twice_gives_the_same_farm_and_writes_once() {
        let repository = FakeFarmRepository::holding(a_farm());

        let (first, _) = use_case(&repository)
            .execute(&auth_context(), 7, an_edit())
            .await
            .expect("first edit");
        let (second, _) = use_case(&repository)
            .execute(&auth_context(), 7, an_edit())
            .await
            .expect("the repeat");

        assert_eq!(second.id(), first.id());
        assert_eq!(second.name(), first.name());
        assert_eq!(second.outline(), first.outline());
        assert_eq!(second.cells(), first.cells());
        assert_eq!(second.updated_at(), first.updated_at());
        assert_eq!(
            repository
                .calls()
                .iter()
                .filter(|call| matches!(call, RepositoryCall::Replace { .. }))
                .count(),
            1,
            "the repeat finds nothing to change"
        );
    }

    #[tokio::test]
    async fn painted_cells_the_new_outline_does_not_touch_are_reported_as_dropped() {
        let repository = FakeFarmRepository::holding(a_farm());

        // The first cell of the larger old outline lies outside the new one.
        let (farm, dropped) = use_case(&repository)
            .execute(
                &auth_context(),
                7,
                EditFarmInput {
                    painted: vec![PaintedCell::new(a_cell_inside(), Crop::Wheat)],
                    ..an_edit()
                },
            )
            .await
            .expect("edit");

        assert_eq!(dropped, vec![a_cell_inside()]);
        assert!(farm.crop_areas().is_empty());
    }

    #[tokio::test]
    async fn an_outline_over_the_cell_limit_is_refused_and_nothing_is_written() {
        let repository = FakeFarmRepository::holding(a_farm());

        let result = EditFarmUseCase::new(Arc::new(repository.clone()), 10)
            .execute(&auth_context(), 7, an_edit())
            .await;

        assert!(matches!(
            result,
            Err(AppError::Farm(FarmError::TooManyCells(10)))
        ));
        assert!(
            !repository
                .calls()
                .iter()
                .any(|call| matches!(call, RepositoryCall::Replace { .. }))
        );
    }

    #[tokio::test]
    async fn only_a_new_name_is_still_an_edit() {
        let before = a_farm();
        let repository = FakeFarmRepository::holding(before.clone());

        let (farm, _) = use_case(&repository)
            .execute(
                &auth_context(),
                7,
                EditFarmInput {
                    name: name("Lower field"),
                    outline: an_outline(),
                    painted: vec![PaintedCell::new(a_cell_inside(), Crop::Wheat)],
                },
            )
            .await
            .expect("edit");

        assert_eq!(farm.name().as_str(), "Lower field");
        assert_eq!(farm.cells().len(), before.cells().len());
        assert_eq!(farm.crop_areas(), before.crop_areas());
    }

    #[tokio::test]
    async fn a_repository_failure_surfaces_instead_of_editing() {
        let repository = FakeFarmRepository::failing();

        assert!(
            use_case(&repository)
                .execute(&auth_context(), 7, an_edit())
                .await
                .is_err()
        );
    }
}
