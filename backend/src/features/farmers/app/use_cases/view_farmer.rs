use std::sync::Arc;

use crate::{
    app::AppError as GlobalAppError,
    features::farmers::app::{AppError, FarmCounter, FarmerRecord, FarmerRepository},
};

/// The dashboard's view of one farmer, by id. The farmer app views its own
/// profile through `ViewProfileUseCase`, by the signed-in phone.
pub struct ViewFarmerUseCase {
    farmers: Arc<dyn FarmerRepository>,
    farms: Arc<dyn FarmCounter>,
}

impl ViewFarmerUseCase {
    pub fn new(farmers: Arc<dyn FarmerRepository>, farms: Arc<dyn FarmCounter>) -> Self {
        Self { farmers, farms }
    }

    pub async fn execute(&self, id: i32) -> Result<FarmerRecord, AppError> {
        let Some(farmer) = self.farmers.find_by_id(id).await? else {
            return Err(GlobalAppError::NotFound.into());
        };

        let farms_count = self.farms.count_for(farmer.phone()).await?;

        Ok(FarmerRecord {
            farmer,
            farms_count,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::features::farmers::app::testing::{Call, Fakes, PHONE};

    fn use_case(fakes: &Fakes) -> ViewFarmerUseCase {
        ViewFarmerUseCase::new(Arc::new(fakes.clone()), Arc::new(fakes.clone()))
    }

    #[tokio::test]
    async fn returns_the_farmer_with_their_farms_count() {
        let fakes = Fakes::new().with_farmer();

        let record = use_case(&fakes).execute(1).await.expect("farmer");

        assert_eq!(record.farmer.phone().as_str(), PHONE);
        assert_eq!(record.farms_count, 2);
        assert_eq!(
            fakes.calls(),
            vec![
                Call::FindFarmerById { id: 1 },
                Call::CountFarms {
                    phone: PHONE.to_string()
                },
            ]
        );
    }

    #[tokio::test]
    async fn is_not_found_when_there_is_no_such_farmer() {
        let fakes = Fakes::new().with_farmer();

        assert!(matches!(
            use_case(&fakes).execute(99).await,
            Err(AppError::GlobalAppError(GlobalAppError::NotFound))
        ));
    }
}
