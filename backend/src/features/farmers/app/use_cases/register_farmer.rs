use std::sync::Arc;

use crate::{
    app::StaffContext,
    features::farmers::{
        app::{AppError, FarmCounter, FarmerRecord, FarmerRepository},
        domain::{Farmer, FarmerName, Language},
    },
    shared::Phone,
};

pub struct RegisterFarmerInput {
    pub phone: Phone,
    pub name: Option<FarmerName>,
    pub language: Language,
}

/// Staff registering a farmer by hand, with no sign-in code. The farmer can
/// sign in with a code later and finds this account.
pub struct RegisterFarmerUseCase {
    farmers: Arc<dyn FarmerRepository>,
    farms: Arc<dyn FarmCounter>,
}

impl RegisterFarmerUseCase {
    pub fn new(farmers: Arc<dyn FarmerRepository>, farms: Arc<dyn FarmCounter>) -> Self {
        Self { farmers, farms }
    }

    pub async fn execute(
        &self,
        actor: &StaffContext,
        input: RegisterFarmerInput,
    ) -> Result<FarmerRecord, AppError> {
        let farmer = Farmer::register(input.phone, input.name, input.language);

        // No lookup first: the insert itself finds out whether the phone is
        // taken, so two requests at the same moment cannot both create.
        let Some(farmer) = self.farmers.create(&farmer).await? else {
            tracing::info!(
                staff_id = *actor.staff_id(),
                "farmer registration by staff refused: the phone already has a farmer"
            );

            return Err(AppError::FarmerAlreadyExists);
        };

        let farms_count = self.farms.count_for(farmer.phone()).await?;

        tracing::info!(
            staff_id = *actor.staff_id(),
            farmer_id = farmer.id().unwrap_or_default(),
            "farmer registered by staff"
        );

        Ok(FarmerRecord {
            farmer,
            farms_count,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::features::farmers::app::testing::{Call, Fakes, PHONE, phone, staff_context};

    fn use_case(fakes: &Fakes) -> RegisterFarmerUseCase {
        RegisterFarmerUseCase::new(Arc::new(fakes.clone()), Arc::new(fakes.clone()))
    }

    fn input() -> RegisterFarmerInput {
        RegisterFarmerInput {
            phone: phone(),
            name: Some(FarmerName::new("Hiwa K.".to_string()).expect("name")),
            language: Language::Kurmanji,
        }
    }

    #[tokio::test]
    async fn registers_a_farmer_without_any_sign_in_code() {
        let fakes = Fakes::new();

        let record = use_case(&fakes)
            .execute(&staff_context(), input())
            .await
            .expect("farmer");

        assert_eq!(*record.farmer.id(), Some(1));
        assert_eq!(*record.farmer.language(), Language::Kurmanji);
        assert!(fakes.stored_challenge().is_none());
        assert_eq!(
            fakes.calls()[0],
            Call::CreateFarmer {
                phone: PHONE.to_string()
            },
            "the create is the first call: no lookup decides before it"
        );
    }

    #[tokio::test]
    async fn a_phone_that_already_has_a_farmer_is_refused_and_left_as_it_is() {
        let fakes = Fakes::new().with_farmer();
        let before = fakes.farmer_created_at();

        let result = use_case(&fakes).execute(&staff_context(), input()).await;

        assert!(matches!(result, Err(AppError::FarmerAlreadyExists)));
        assert_eq!(fakes.farmer_created_at(), before);
    }
}
