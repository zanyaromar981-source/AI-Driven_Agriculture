use std::sync::Arc;

use crate::{
    app::{AppError as GlobalAppError, StaffContext},
    features::farms::{
        app::{
            AppError, FarmerDirectory,
            use_cases::{RegisterFarmInput, RegisterFarmUseCase},
        },
        domain::{Farm, GridCell},
    },
    shared::Phone,
};

/// Staff registering a farm on behalf of a farmer. The registration itself
/// is the farmer's own (`RegisterFarmUseCase`), so the outline checks, the
/// cell limit, the per-farmer farm limit and the idempotency key all apply
/// unchanged; this adds only that the farmer must exist.
pub struct RegisterFarmForFarmerUseCase {
    farmers: Arc<dyn FarmerDirectory>,
    registration: Arc<RegisterFarmUseCase>,
}

impl RegisterFarmForFarmerUseCase {
    pub fn new(farmers: Arc<dyn FarmerDirectory>, registration: Arc<RegisterFarmUseCase>) -> Self {
        Self {
            farmers,
            registration,
        }
    }

    pub async fn execute(
        &self,
        actor: &StaffContext,
        owner: Phone,
        input: RegisterFarmInput,
    ) -> Result<(Farm, Vec<GridCell>), AppError> {
        if !self.farmers.is_registered(&owner).await? {
            tracing::info!(
                staff_id = *actor.staff_id(),
                "farm registration by staff refused: no farmer has that phone"
            );

            return Err(GlobalAppError::NotFound.into());
        }

        let (farm, dropped_cells) = self.registration.execute_for_owner(&owner, input).await?;

        tracing::info!(
            staff_id = *actor.staff_id(),
            farm_id = farm.id().unwrap_or_default(),
            "farm registered by staff for a farmer"
        );

        Ok((farm, dropped_cells))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::features::farms::{
        app::testing::{
            FakeFarmRepository, FakeFarmerDirectory, FakePlaceLocator, MAX_CELLS, RepositoryCall,
            an_outline, staff_context, the_place,
        },
        domain::FarmName,
    };

    const FARMER: &str = "+9647507654321";
    const MAX: u64 = 3;

    fn farmer() -> Phone {
        Phone::new(FARMER.to_string()).expect("phone")
    }

    fn input() -> RegisterFarmInput {
        RegisterFarmInput {
            name: FarmName::new("Upper field".to_string()).expect("name"),
            outline: an_outline(),
            painted: vec![],
            idempotency_key: None,
            created_offline_at: None,
        }
    }

    fn use_case(
        farmers: &FakeFarmerDirectory,
        repository: &FakeFarmRepository,
    ) -> RegisterFarmForFarmerUseCase {
        RegisterFarmForFarmerUseCase::new(
            Arc::new(farmers.clone()),
            Arc::new(RegisterFarmUseCase::new(
                Arc::new(repository.clone()),
                Arc::new(FakePlaceLocator::new()),
                MAX,
                MAX_CELLS,
            )),
        )
    }

    #[tokio::test]
    async fn the_farm_belongs_to_the_named_farmer_not_to_the_staff_member() {
        let farmers = FakeFarmerDirectory::knowing_everyone();
        let repository = FakeFarmRepository::new();

        let (farm, _) = use_case(&farmers, &repository)
            .execute(&staff_context(), farmer(), input())
            .await
            .expect("farm");

        assert_eq!(farm.owner().as_str(), FARMER);
        assert_eq!(farmers.asked(), vec![FARMER.to_string()]);
    }

    #[tokio::test]
    async fn a_farm_staff_register_gets_its_place_like_the_farmers_own() {
        let farmers = FakeFarmerDirectory::knowing_everyone();
        let repository = FakeFarmRepository::new();

        let (farm, _) = use_case(&farmers, &repository)
            .execute(&staff_context(), farmer(), input())
            .await
            .expect("farm");

        assert_eq!(farm.place(), &Some(the_place()));
    }

    #[tokio::test]
    async fn a_phone_no_farmer_has_is_not_found_and_nothing_is_written() {
        let farmers = FakeFarmerDirectory::knowing_no_one();
        let repository = FakeFarmRepository::new();

        let result = use_case(&farmers, &repository)
            .execute(&staff_context(), farmer(), input())
            .await;

        assert!(matches!(
            result,
            Err(AppError::GlobalAppError(GlobalAppError::NotFound))
        ));
        assert!(
            repository.calls().is_empty(),
            "a farm must not be created for a farmer who does not exist"
        );
    }

    #[tokio::test]
    async fn the_farmers_own_farm_limit_applies_to_staff_too() {
        let farmers = FakeFarmerDirectory::knowing_everyone();
        let repository = FakeFarmRepository::owning(MAX);

        let result = use_case(&farmers, &repository)
            .execute(&staff_context(), farmer(), input())
            .await;

        assert!(matches!(result, Err(AppError::MaxFarmsPerUserReached(MAX))));
        assert_eq!(
            repository.calls(),
            vec![RepositoryCall::CountByOwner {
                owner: FARMER.to_string()
            }],
            "the limit is counted for the farmer, and nothing is written past it"
        );
    }
}
