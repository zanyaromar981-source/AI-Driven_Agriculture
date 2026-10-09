use std::sync::Arc;

use chrono::Utc;

use crate::{
    app::{AppError as GlobalAppError, StaffContext},
    features::farmers::{
        app::{AppError, FarmCounter, FarmerRecord, FarmerRepository},
        domain::{FarmerName, Language},
    },
};

/// What staff may change about a farmer. There is no phone here: the phone
/// is the account and cannot change.
pub struct EditFarmerInput {
    pub name: Option<FarmerName>,
    pub language: Language,
}

pub struct EditFarmerUseCase {
    farmers: Arc<dyn FarmerRepository>,
    farms: Arc<dyn FarmCounter>,
}

impl EditFarmerUseCase {
    pub fn new(farmers: Arc<dyn FarmerRepository>, farms: Arc<dyn FarmCounter>) -> Self {
        Self { farmers, farms }
    }

    pub async fn execute(
        &self,
        actor: &StaffContext,
        id: i32,
        input: EditFarmerInput,
    ) -> Result<FarmerRecord, AppError> {
        let Some(farmer) = self
            .farmers
            .update_by_id(id, input.name.as_ref(), input.language, Utc::now())
            .await?
        else {
            tracing::info!(
                staff_id = *actor.staff_id(),
                farmer_id = id,
                "farmer edit by staff refused: no such farmer"
            );

            return Err(GlobalAppError::NotFound.into());
        };

        let farms_count = self.farms.count_for(farmer.phone()).await?;

        tracing::info!(
            staff_id = *actor.staff_id(),
            farmer_id = id,
            "farmer edited by staff"
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
    use crate::features::farmers::app::testing::{Call, Fakes, PHONE, staff_context};

    fn use_case(fakes: &Fakes) -> EditFarmerUseCase {
        EditFarmerUseCase::new(Arc::new(fakes.clone()), Arc::new(fakes.clone()))
    }

    fn input(name: Option<&str>) -> EditFarmerInput {
        EditFarmerInput {
            name: name.map(|name| FarmerName::new(name.to_string()).expect("name")),
            language: Language::English,
        }
    }

    #[tokio::test]
    async fn replaces_the_name_and_language_and_keeps_the_phone() {
        let fakes = Fakes::new().with_farmer();

        let record = use_case(&fakes)
            .execute(&staff_context(), 1, input(Some("Hiwa K.")))
            .await
            .expect("farmer");

        assert_eq!(*record.farmer.language(), Language::English);
        assert_eq!(record.farmer.phone().as_str(), PHONE);
        assert_eq!(
            fakes.calls()[0],
            Call::UpdateFarmerById { id: 1 },
            "the update is the first call: no lookup decides before it"
        );
    }

    #[tokio::test]
    async fn a_null_name_clears_it() {
        let fakes = Fakes::new().with_farmer();

        let record = use_case(&fakes)
            .execute(&staff_context(), 1, input(None))
            .await
            .expect("farmer");

        assert!(record.farmer.name().is_none());
    }

    #[tokio::test]
    async fn is_not_found_when_there_is_no_such_farmer() {
        let fakes = Fakes::new();

        assert!(matches!(
            use_case(&fakes)
                .execute(&staff_context(), 1, input(None))
                .await,
            Err(AppError::GlobalAppError(GlobalAppError::NotFound))
        ));
    }
}
