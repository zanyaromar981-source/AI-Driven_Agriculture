use std::sync::Arc;

use crate::{
    app::{AppError as GlobalAppError, AuthContext},
    features::farms::{
        app::{AppError, CropDirectory, FarmRepository},
        domain::{Farm, GridCell, PaintedCell},
    },
};

pub struct RepaintFarmCellsInput {
    pub painted: Vec<PaintedCell>,
}

pub struct RepaintFarmCellsUseCase {
    repository: Arc<dyn FarmRepository>,
    crops: Arc<dyn CropDirectory>,
}

impl RepaintFarmCellsUseCase {
    pub fn new(repository: Arc<dyn FarmRepository>, crops: Arc<dyn CropDirectory>) -> Self {
        Self { repository, crops }
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

        // Only the cells that change crop are new data. The list of crops
        // is read once for the whole repaint, and not at all when the
        // repaint only unpaints cells or repeats what is stored.
        let introduced = farm.crops_introduced_by(&input.painted);

        if !introduced.is_empty() {
            self.crops
                .active()
                .await?
                .allow(introduced)
                .inspect_err(|error| {
                    tracing::info!(farm_id = id, %error, "repaint refused: a crop is not in use");
                })?;
        }

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
            FakeCropDirectory, FakeFarmRepository, OWNER, RepositoryCall, a_cell_inside,
            a_cell_outside, a_farm, auth_context,
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
        let use_case = RepaintFarmCellsUseCase::new(
            Arc::new(repository.clone()),
            Arc::new(FakeCropDirectory::seeded()),
        );

        let (farm, dropped) = use_case
            .execute(
                &auth_context(),
                7,
                input(a_cell_inside(), Crop::of("barley")),
            )
            .await
            .expect("repaint");

        assert_eq!(farm.crop_areas()[0].crop(), Crop::of("barley"));
        assert!(dropped.is_empty());
        assert!(repository.calls().contains(&RepositoryCall::Update));
    }

    #[tokio::test]
    async fn is_not_found_when_the_user_does_not_own_it() {
        let repository = FakeFarmRepository::new();
        let use_case = RepaintFarmCellsUseCase::new(
            Arc::new(repository.clone()),
            Arc::new(FakeCropDirectory::seeded()),
        );

        let result = use_case
            .execute(
                &auth_context(),
                7,
                input(a_cell_inside(), Crop::of("barley")),
            )
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
        let use_case = RepaintFarmCellsUseCase::new(
            Arc::new(repository.clone()),
            Arc::new(FakeCropDirectory::seeded()),
        );

        let _ = use_case
            .execute(
                &auth_context(),
                7,
                input(a_cell_inside(), Crop::of("barley")),
            )
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
        let use_case = RepaintFarmCellsUseCase::new(
            Arc::new(repository.clone()),
            Arc::new(FakeCropDirectory::seeded()),
        );

        let (farm, dropped) = use_case
            .execute(
                &auth_context(),
                7,
                input(a_cell_outside(), Crop::of("barley")),
            )
            .await
            .expect("repaint");

        assert_eq!(dropped, vec![a_cell_outside()]);
        assert_eq!(
            farm.crop_areas()[0].crop(),
            Crop::of("wheat"),
            "the farm keeps the crops it had"
        );
    }

    fn use_case(
        repository: &FakeFarmRepository,
        crops: &FakeCropDirectory,
    ) -> RepaintFarmCellsUseCase {
        RepaintFarmCellsUseCase::new(Arc::new(repository.clone()), Arc::new(crops.clone()))
    }

    #[tokio::test]
    async fn a_crop_that_is_unknown_or_switched_off_is_refused_by_name_and_nothing_is_written() {
        let repository = FakeFarmRepository::holding(a_farm());
        let crops = FakeCropDirectory::with(&["wheat"]);

        let result = use_case(&repository, &crops)
            .execute(&auth_context(), 7, input(a_cell_inside(), Crop::of("rice")))
            .await;

        assert!(matches!(
            result,
            Err(AppError::Farm(crate::features::farms::domain::FarmError::UnknownCrop(code)))
                if code == "rice"
        ));
        assert!(!repository.calls().contains(&RepositoryCall::Update));
    }

    #[tokio::test]
    async fn a_cell_keeps_a_crop_that_was_switched_off_after_it_was_painted() {
        // The stored farm has wheat on this cell, and wheat is off now.
        let repository = FakeFarmRepository::holding(a_farm());
        let crops = FakeCropDirectory::with(&["barley"]);

        let (farm, _) = use_case(&repository, &crops)
            .execute(
                &auth_context(),
                7,
                input(a_cell_inside(), Crop::of("wheat")),
            )
            .await
            .expect("sending the crop a cell already has changes nothing");

        assert_eq!(farm.crop_areas()[0].crop(), Crop::of("wheat"));
        assert_eq!(crops.asked(), 0, "nothing new is being stored");
    }

    #[tokio::test]
    async fn unpainting_a_cell_needs_no_crop_list() {
        let repository = FakeFarmRepository::holding(a_farm());
        let crops = FakeCropDirectory::failing();

        let (farm, _) = use_case(&repository, &crops)
            .execute(&auth_context(), 7, input(a_cell_inside(), Crop::EMPTY))
            .await
            .expect("repaint");

        assert!(farm.crop_areas().is_empty());
        assert_eq!(crops.asked(), 0);
    }

    #[tokio::test]
    async fn the_crop_list_is_read_once_for_a_repaint_of_many_cells() {
        let stored = a_farm();
        let painted = stored
            .cells()
            .iter()
            .map(|cell| PaintedCell::new(cell.position(), Crop::of("barley")))
            .collect::<Vec<_>>();
        assert!(painted.len() > 1, "the point is many cells");
        let repository = FakeFarmRepository::holding(stored);
        let crops = FakeCropDirectory::seeded();

        use_case(&repository, &crops)
            .execute(&auth_context(), 7, RepaintFarmCellsInput { painted })
            .await
            .expect("repaint");

        assert_eq!(crops.asked(), 1);
    }

    #[tokio::test]
    async fn when_the_crop_list_cannot_be_read_nothing_is_repainted() {
        let repository = FakeFarmRepository::holding(a_farm());

        let result = use_case(&repository, &FakeCropDirectory::failing())
            .execute(
                &auth_context(),
                7,
                input(a_cell_inside(), Crop::of("barley")),
            )
            .await;

        assert!(result.is_err());
        assert!(!repository.calls().contains(&RepositoryCall::Update));
    }
}
