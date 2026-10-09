use std::sync::Arc;

use crate::{
    app::StaffContext,
    features::farmers::app::{AppError, FarmRemover, FarmerRepository},
};

pub struct RemoveFarmerUseCase {
    farmers: Arc<dyn FarmerRepository>,
    farms: Arc<dyn FarmRemover>,
}

impl RemoveFarmerUseCase {
    pub fn new(farmers: Arc<dyn FarmerRepository>, farms: Arc<dyn FarmRemover>) -> Self {
        Self { farmers, farms }
    }

    /// Removes the farmer, the sign-in challenge their phone has open and
    /// all their farms. Removing a farmer who is not there succeeds.
    ///
    /// The farms belong to another feature and cannot share a transaction
    /// with the farmer, so they go first: if that step fails the farmer is
    /// still there and the delete can simply be sent again. The other order
    /// could leave farms behind with no farmer to find them by.
    pub async fn execute(&self, actor: &StaffContext, id: i32) -> Result<(), AppError> {
        // This read does not decide anything: it only finds the phone the
        // farms are kept under. Both deletes are safe to repeat.
        let Some(farmer) = self.farmers.find_by_id(id).await? else {
            tracing::info!(
                staff_id = *actor.staff_id(),
                farmer_id = id,
                "farmer already gone: nothing to remove"
            );

            return Ok(());
        };

        let farms_removed = self.farms.remove_all_for(farmer.phone()).await?;

        self.farmers.delete_with_challenge(id).await?;

        tracing::info!(
            staff_id = *actor.staff_id(),
            farmer_id = id,
            farms_removed,
            "farmer removed by staff"
        );

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::features::farmers::app::testing::{Call, Fakes, PHONE, staff_context};

    fn use_case(fakes: &Fakes) -> RemoveFarmerUseCase {
        RemoveFarmerUseCase::new(Arc::new(fakes.clone()), Arc::new(fakes.clone()))
    }

    #[tokio::test]
    async fn removes_the_farms_first_then_the_farmer_and_their_challenge() {
        let fakes = Fakes::with_open_challenge("123456").with_farmer();

        use_case(&fakes)
            .execute(&staff_context(), 1)
            .await
            .expect("removed");

        assert_eq!(
            fakes.calls(),
            vec![
                Call::FindFarmerById { id: 1 },
                Call::RemoveFarms {
                    phone: PHONE.to_string()
                },
                Call::DeleteFarmerWithChallenge { id: 1 },
            ]
        );
        assert!(!fakes.has_farmer());
        assert!(fakes.stored_challenge().is_none());
    }

    #[tokio::test]
    async fn removing_a_farmer_who_is_already_gone_succeeds_and_touches_nothing() {
        let fakes = Fakes::new();

        assert!(use_case(&fakes).execute(&staff_context(), 1).await.is_ok());
        assert_eq!(fakes.calls(), vec![Call::FindFarmerById { id: 1 }]);
    }

    #[tokio::test]
    async fn the_farmer_stays_when_their_farms_could_not_be_removed() {
        let fakes = Fakes::new().with_farmer().failing_to_remove_farms();

        assert!(use_case(&fakes).execute(&staff_context(), 1).await.is_err());
        assert!(
            fakes.has_farmer(),
            "the delete must be repeatable, so the farmer may not go before the farms"
        );
    }
}
