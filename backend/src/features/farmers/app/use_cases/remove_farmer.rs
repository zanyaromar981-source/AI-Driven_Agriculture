use std::sync::Arc;

use crate::{
    app::{AuthContext, StaffContext},
    features::farmers::{
        app::{AppError, FarmRemover, FarmerDataRemover, FarmerRepository},
        domain::Farmer,
    },
};

pub struct RemoveFarmerUseCase {
    farmers: Arc<dyn FarmerRepository>,
    farms: Arc<dyn FarmRemover>,
    attached: Arc<dyn FarmerDataRemover>,
}

impl RemoveFarmerUseCase {
    pub fn new(
        farmers: Arc<dyn FarmerRepository>,
        farms: Arc<dyn FarmRemover>,
        attached: Arc<dyn FarmerDataRemover>,
    ) -> Self {
        Self {
            farmers,
            farms,
            attached,
        }
    }

    /// Removes the farmer, the sign-in challenge their phone has open, all
    /// their farms, the alerts of those farms and the phones registered for
    /// pushes. Removing a farmer who is not there succeeds.
    ///
    /// The farms belong to another feature and cannot share a transaction
    /// with the farmer, so they go first: if that step fails the farmer is
    /// still there and the delete can simply be sent again. The other order
    /// could leave farms behind with no farmer to find them by.
    pub async fn execute(&self, actor: &StaffContext, id: i32) -> Result<(), AppError> {
        // This read does not decide anything: it only finds the phone the
        // farms are kept under. Every delete is safe to repeat.
        let Some(farmer) = self.farmers.find_by_id(id).await? else {
            tracing::info!(
                staff_id = *actor.staff_id(),
                farmer_id = id,
                "farmer already gone: nothing to remove"
            );

            return Ok(());
        };

        let farms_removed = self.remove(&farmer, id).await?;

        tracing::info!(
            staff_id = *actor.staff_id(),
            farmer_id = id,
            farms_removed,
            "farmer removed by staff"
        );

        Ok(())
    }

    /// The same removal, asked for by the farmer from the app. Their token
    /// stops working the moment the farmer is gone, so the app is signed
    /// out at once.
    pub async fn execute_for_self(&self, auth_context: &AuthContext) -> Result<(), AppError> {
        let Some(farmer) = self
            .farmers
            .find_by_phone(auth_context.user().phone())
            .await?
        else {
            tracing::info!("account already gone: nothing to remove");

            return Ok(());
        };

        let Some(id) = *farmer.id() else {
            return Ok(());
        };

        let farms_removed = self.remove(&farmer, id).await?;

        tracing::info!(
            farmer_id = id,
            farms_removed,
            "farmer removed their account"
        );

        Ok(())
    }

    /// The alerts are found through the farms, so they go before the farms;
    /// the farmer goes last, for the reason given on `execute`.
    async fn remove(&self, farmer: &Farmer, id: i32) -> Result<u64, AppError> {
        self.attached.remove_all_for(farmer.phone()).await?;

        let farms_removed = self.farms.remove_all_for(farmer.phone()).await?;

        self.farmers.delete_with_challenge(id).await?;

        Ok(farms_removed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::features::farmers::app::testing::{Call, Fakes, PHONE, auth_context, staff_context};

    fn use_case(fakes: &Fakes) -> RemoveFarmerUseCase {
        RemoveFarmerUseCase::new(
            Arc::new(fakes.clone()),
            Arc::new(fakes.clone()),
            Arc::new(fakes.clone()),
        )
    }

    #[tokio::test]
    async fn removes_the_alert_data_then_the_farms_then_the_farmer_and_their_challenge() {
        let fakes = Fakes::with_open_challenge("123456").with_farmer();

        use_case(&fakes)
            .execute(&staff_context(), 1)
            .await
            .expect("removed");

        assert_eq!(
            fakes.calls(),
            vec![
                Call::FindFarmerById { id: 1 },
                Call::RemoveFarmerData {
                    phone: PHONE.to_string()
                },
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

    #[tokio::test]
    async fn a_farmer_deleting_their_own_account_removes_exactly_what_staff_would() {
        let fakes = Fakes::with_open_challenge("123456").with_farmer();

        use_case(&fakes)
            .execute_for_self(&auth_context())
            .await
            .expect("removed");

        assert_eq!(
            fakes.calls(),
            vec![
                Call::FindFarmer {
                    phone: PHONE.to_string()
                },
                Call::RemoveFarmerData {
                    phone: PHONE.to_string()
                },
                Call::RemoveFarms {
                    phone: PHONE.to_string()
                },
                Call::DeleteFarmerWithChallenge { id: 1 },
            ],
            "found by the signed-in phone, never by an id the app sends"
        );
        assert!(!fakes.has_farmer());
    }

    #[tokio::test]
    async fn deleting_an_account_that_is_already_gone_succeeds() {
        let fakes = Fakes::new();

        assert!(
            use_case(&fakes)
                .execute_for_self(&auth_context())
                .await
                .is_ok()
        );
        assert_eq!(fakes.calls().len(), 1);
    }
}
