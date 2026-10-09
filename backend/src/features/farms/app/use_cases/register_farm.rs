use std::sync::Arc;

use chrono::{DateTime, Utc};

use crate::{
    app::AuthContext,
    features::farms::{
        app::{AppError, CropDirectory, FarmRepository, PlaceLocator},
        domain::{Farm, FarmName, GridCell, IdempotencyKey, Outline, PaintedCell},
    },
    shared::Phone,
};

pub struct RegisterFarmInput {
    pub name: FarmName,
    pub outline: Outline,
    pub painted: Vec<PaintedCell>,
    pub idempotency_key: Option<IdempotencyKey>,
    pub created_offline_at: Option<DateTime<Utc>>,
}

pub struct RegisterFarmUseCase {
    repository: Arc<dyn FarmRepository>,
    places: Arc<dyn PlaceLocator>,
    crops: Arc<dyn CropDirectory>,
    max_farms_per_user: u64,
    max_cells_per_farm: usize,
}

impl RegisterFarmUseCase {
    pub fn new(
        repository: Arc<dyn FarmRepository>,
        places: Arc<dyn PlaceLocator>,
        crops: Arc<dyn CropDirectory>,
        max_farms_per_user: u64,
        max_cells_per_farm: usize,
    ) -> Self {
        Self {
            repository,
            places,
            crops,
            max_farms_per_user,
            max_cells_per_farm,
        }
    }

    /// Returns the registered farm and the painted cells that were dropped
    /// because they fall outside its outline. An upload that repeats the
    /// idempotency key of an earlier one returns that earlier farm.
    pub async fn execute(
        &self,
        auth_context: &AuthContext,
        input: RegisterFarmInput,
    ) -> Result<(Farm, Vec<GridCell>), AppError> {
        self.execute_for_owner(auth_context.user().phone(), input)
            .await
    }

    /// The same registration with the owner named by the caller. The farmer
    /// app never comes this way: `execute` takes the owner from the token.
    /// It exists so that a farm staff register for a farmer passes exactly
    /// the rules the farmer's own upload passes.
    pub async fn execute_for_owner(
        &self,
        owner: &Phone,
        input: RegisterFarmInput,
    ) -> Result<(Farm, Vec<GridCell>), AppError> {
        if let Some(key) = &input.idempotency_key
            && let Some(existing) = self
                .repository
                .find_by_idempotency_key_and_owner(key, owner)
                .await?
        {
            tracing::info!(
                farm_id = existing.id().unwrap_or_default(),
                "registration repeated: returning the farm already created"
            );

            return Ok((existing, Vec::new()));
        }

        // A new farm is new data, so every crop painted on it must be one
        // staff have switched on. The list is read once for the whole farm,
        // and not at all for a farm with nothing painted. A repeat of an
        // earlier upload never gets here: that farm was checked when it was
        // made, and a crop switched off since does not undo it.
        let planted: Vec<_> = input
            .painted
            .iter()
            .map(|cell| cell.crop())
            .filter(|crop| !crop.is_empty())
            .collect();

        if !planted.is_empty() {
            self.crops.active().await?.allow(planted).inspect_err(
                |error| tracing::info!(%error, "registration refused: a crop is not in use"),
            )?;
        }

        let owned = self.repository.count_by_owner(owner).await?;

        if owned >= self.max_farms_per_user {
            tracing::info!(
                owned,
                max = self.max_farms_per_user,
                "registration refused: farm quota reached"
            );

            return Err(AppError::MaxFarmsPerUserReached(self.max_farms_per_user));
        }

        let (mut farm, dropped_cells) = Farm::new(
            input.name,
            owner.clone(),
            input.outline,
            input.painted,
            input.idempotency_key,
            input.created_offline_at,
            self.max_cells_per_farm,
        )?;

        // A farm is stored with its place or not at all: one saved without
        // it because the lookup failed would be missing from every report
        // by district until someone noticed.
        let (lat, lon) = farm.outline().centroid();
        farm.place_at(self.places.locate(lat, lon).await?);

        let registered = match self.repository.create(&farm).await {
            Ok(registered) => registered,
            Err(error) => {
                // Two uploads with the same key can both pass the lookup
                // above; the database lets only one in. The other is the
                // same farm arriving twice, so it gets the farm that won.
                if let Some(key) = farm.idempotency_key()
                    && let Some(existing) = self
                        .repository
                        .find_by_idempotency_key_and_owner(key, owner)
                        .await?
                {
                    tracing::info!(
                        farm_id = existing.id().unwrap_or_default(),
                        "registration raced its own repeat: returning the farm already created"
                    );

                    return Ok((existing, Vec::new()));
                }

                return Err(error);
            }
        };

        tracing::info!(
            farm_id = registered.id().unwrap_or_default(),
            cells = registered.cells().len(),
            dropped_cells = dropped_cells.len(),
            placed = registered.place().is_some(),
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
            FakeCropDirectory, FakeFarmRepository, FakePlaceLocator, MAX_CELLS, OWNER,
            RepositoryCall, a_cell_inside, a_cell_outside, an_outline, an_outline_outside,
            auth_context, the_place,
        },
        domain::{Crop, FarmError},
    };

    fn a_key() -> IdempotencyKey {
        IdempotencyKey::new("upload-1".to_string()).expect("key")
    }

    const MAX: u64 = 3;

    fn input(painted: Vec<PaintedCell>) -> RegisterFarmInput {
        RegisterFarmInput {
            name: FarmName::new("Upper field".to_string()).expect("name"),
            outline: an_outline(),
            painted,
            idempotency_key: None,
            created_offline_at: None,
        }
    }

    async fn register(
        repository: FakeFarmRepository,
        max_cells: usize,
        painted: Vec<PaintedCell>,
    ) -> (Result<(Farm, Vec<GridCell>), AppError>, FakeFarmRepository) {
        let use_case = RegisterFarmUseCase::new(
            Arc::new(repository.clone()),
            Arc::new(FakePlaceLocator::new()),
            Arc::new(FakeCropDirectory::seeded()),
            MAX,
            max_cells,
        );
        let result = use_case.execute(&auth_context(), input(painted)).await;

        (result, repository)
    }

    #[tokio::test]
    async fn the_farm_is_stored_with_the_place_its_centre_lies_in() {
        let places = FakePlaceLocator::new();
        let use_case = RegisterFarmUseCase::new(
            Arc::new(FakeFarmRepository::new()),
            Arc::new(places.clone()),
            Arc::new(FakeCropDirectory::seeded()),
            MAX,
            MAX_CELLS,
        );

        let (farm, _) = use_case
            .execute(&auth_context(), input(vec![]))
            .await
            .expect("farm");

        assert_eq!(farm.place(), &Some(the_place()));
        assert_eq!(
            places.asked(),
            vec![an_outline().centroid()],
            "the centre of the outline is what is looked up, as (lat, lon)"
        );
    }

    #[tokio::test]
    async fn a_farm_outside_every_place_is_stored_with_none() {
        let repository = FakeFarmRepository::new();
        let use_case = RegisterFarmUseCase::new(
            Arc::new(repository.clone()),
            Arc::new(FakePlaceLocator::new()),
            Arc::new(FakeCropDirectory::seeded()),
            MAX,
            MAX_CELLS,
        );

        let (farm, _) = use_case
            .execute(
                &auth_context(),
                RegisterFarmInput {
                    outline: an_outline_outside(),
                    ..input(vec![])
                },
            )
            .await
            .expect("farm");

        assert_eq!(farm.place(), &None);
        assert!(repository.calls().contains(&RepositoryCall::Create));
    }

    #[tokio::test]
    async fn a_farm_whose_place_cannot_be_looked_up_is_not_written() {
        let repository = FakeFarmRepository::new();
        let use_case = RegisterFarmUseCase::new(
            Arc::new(repository.clone()),
            Arc::new(FakePlaceLocator::failing()),
            Arc::new(FakeCropDirectory::seeded()),
            MAX,
            MAX_CELLS,
        );

        let result = use_case.execute(&auth_context(), input(vec![])).await;

        assert!(result.is_err());
        assert!(!repository.calls().contains(&RepositoryCall::Create));
    }

    #[tokio::test]
    async fn a_refused_or_repeated_upload_looks_no_place_up() {
        let places = FakePlaceLocator::new();

        let full = RegisterFarmUseCase::new(
            Arc::new(FakeFarmRepository::owning(MAX)),
            Arc::new(places.clone()),
            Arc::new(FakeCropDirectory::seeded()),
            MAX,
            MAX_CELLS,
        );
        assert!(full.execute(&auth_context(), input(vec![])).await.is_err());

        let repeated = RegisterFarmUseCase::new(
            Arc::new(FakeFarmRepository::holding(
                crate::features::farms::app::testing::a_farm(),
            )),
            Arc::new(places.clone()),
            Arc::new(FakeCropDirectory::seeded()),
            MAX,
            MAX_CELLS,
        );
        let again = RegisterFarmInput {
            idempotency_key: Some(a_key()),
            ..input(vec![])
        };
        assert!(repeated.execute(&auth_context(), again).await.is_ok());

        assert!(places.asked().is_empty());
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
                PaintedCell::new(a_cell_inside(), Crop::of("wheat")),
                PaintedCell::new(a_cell_outside(), Crop::of("wheat")),
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
    async fn a_repeated_upload_returns_the_farm_already_created() {
        let repository =
            FakeFarmRepository::holding(crate::features::farms::app::testing::a_farm());
        let use_case = RegisterFarmUseCase::new(
            Arc::new(repository.clone()),
            Arc::new(FakePlaceLocator::new()),
            Arc::new(FakeCropDirectory::seeded()),
            MAX,
            MAX_CELLS,
        );

        let (farm, dropped) = use_case
            .execute(
                &auth_context(),
                RegisterFarmInput {
                    idempotency_key: Some(a_key()),
                    ..input(vec![])
                },
            )
            .await
            .expect("farm");

        assert_eq!(*farm.id(), Some(7));
        assert!(dropped.is_empty());
        assert_eq!(
            repository.calls(),
            vec![RepositoryCall::FindByIdempotencyKeyAndOwner {
                key: "upload-1".to_string(),
                owner: OWNER.to_string(),
            }],
            "a repeat must not count the quota or write again"
        );
    }

    #[tokio::test]
    async fn a_first_upload_with_a_key_is_registered() {
        let repository = FakeFarmRepository::new();
        let use_case = RegisterFarmUseCase::new(
            Arc::new(repository.clone()),
            Arc::new(FakePlaceLocator::new()),
            Arc::new(FakeCropDirectory::seeded()),
            MAX,
            MAX_CELLS,
        );

        let result = use_case
            .execute(
                &auth_context(),
                RegisterFarmInput {
                    idempotency_key: Some(a_key()),
                    ..input(vec![])
                },
            )
            .await;

        assert!(result.is_ok());
        assert!(repository.calls().contains(&RepositoryCall::Create));
    }

    #[tokio::test]
    async fn a_repository_failure_surfaces_instead_of_registering() {
        let (result, repository) = register(FakeFarmRepository::failing(), MAX_CELLS, vec![]).await;

        assert!(result.is_err());
        assert!(!repository.calls().contains(&RepositoryCall::Create));
    }

    fn use_case(repository: &FakeFarmRepository, crops: &FakeCropDirectory) -> RegisterFarmUseCase {
        RegisterFarmUseCase::new(
            Arc::new(repository.clone()),
            Arc::new(FakePlaceLocator::new()),
            Arc::new(crops.clone()),
            MAX,
            MAX_CELLS,
        )
    }

    #[tokio::test]
    async fn a_crop_staff_added_later_can_be_painted_on_a_new_farm() {
        let repository = FakeFarmRepository::new();
        let crops = FakeCropDirectory::with(&["wheat", "rice"]);

        let (farm, _) = use_case(&repository, &crops)
            .execute(
                &auth_context(),
                input(vec![PaintedCell::new(a_cell_inside(), Crop::of("rice"))]),
            )
            .await
            .expect("farm");

        assert_eq!(farm.crop_areas()[0].crop(), Crop::of("rice"));
    }

    #[tokio::test]
    async fn a_crop_that_is_unknown_or_switched_off_is_refused_by_name_and_nothing_is_stored() {
        let repository = FakeFarmRepository::new();
        // Rice is not in the list: it was never added, or staff switched it off.
        let crops = FakeCropDirectory::with(&["wheat"]);

        let result = use_case(&repository, &crops)
            .execute(
                &auth_context(),
                input(vec![
                    PaintedCell::new(a_cell_inside(), Crop::of("wheat")),
                    PaintedCell::new(a_cell_outside(), Crop::of("rice")),
                ]),
            )
            .await;

        assert!(matches!(
            result,
            Err(AppError::Farm(FarmError::UnknownCrop(code))) if code == "rice"
        ));
        assert!(
            repository.calls().is_empty(),
            "the crops are checked before anything is counted or written"
        );
    }

    #[tokio::test]
    async fn the_crop_list_is_read_once_however_many_cells_are_painted() {
        let repository = FakeFarmRepository::new();
        let crops = FakeCropDirectory::seeded();
        let painted = an_outline()
            .cells(MAX_CELLS)
            .expect("cells")
            .into_iter()
            .map(|cell| PaintedCell::new(cell.position(), Crop::of("wheat")))
            .collect::<Vec<_>>();
        assert!(painted.len() > 1, "the point is many cells");

        use_case(&repository, &crops)
            .execute(&auth_context(), input(painted))
            .await
            .expect("farm");

        assert_eq!(crops.asked(), 1);
    }

    #[tokio::test]
    async fn a_farm_with_nothing_planted_needs_no_crop_list() {
        let repository = FakeFarmRepository::new();
        let crops = FakeCropDirectory::failing();

        use_case(&repository, &crops)
            .execute(
                &auth_context(),
                input(vec![PaintedCell::new(a_cell_inside(), Crop::EMPTY)]),
            )
            .await
            .expect("empty is the farms feature's own word, not a crop to look up");

        assert_eq!(crops.asked(), 0);
    }

    #[tokio::test]
    async fn when_the_crop_list_cannot_be_read_the_farm_is_not_stored() {
        let repository = FakeFarmRepository::new();

        let result = use_case(&repository, &FakeCropDirectory::failing())
            .execute(
                &auth_context(),
                input(vec![PaintedCell::new(a_cell_inside(), Crop::of("wheat"))]),
            )
            .await;

        assert!(result.is_err());
        assert!(!repository.calls().contains(&RepositoryCall::Create));
    }

    #[tokio::test]
    async fn a_repeated_upload_returns_its_farm_even_after_its_crop_was_switched_off() {
        let existing = crate::features::farms::app::testing::a_farm();
        let repository = FakeFarmRepository::holding(existing);
        let crops = FakeCropDirectory::with(&[]);

        let mut repeat = input(vec![PaintedCell::new(a_cell_inside(), Crop::of("wheat"))]);
        repeat.idempotency_key = Some(a_key());

        let (farm, _) = use_case(&repository, &crops)
            .execute(&auth_context(), repeat)
            .await
            .expect("the farm made the first time");

        assert_eq!(farm.id(), &Some(7));
        assert_eq!(crops.asked(), 0, "nothing new is being stored");
    }
}
