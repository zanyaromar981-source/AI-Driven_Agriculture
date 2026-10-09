use std::sync::Arc;

use crate::{
    features::farmers::{
        app::{AppError, FarmerRepository},
        domain::{Farmer, Language},
    },
    shared::Phone,
};

/// Makes sure a phone has a farmer, for a token printed on the command
/// line: a farmer's token is refused while its farmer does not exist.
pub struct EnsureFarmerUseCase {
    farmers: Arc<dyn FarmerRepository>,
}

impl EnsureFarmerUseCase {
    pub fn new(farmers: Arc<dyn FarmerRepository>) -> Self {
        Self { farmers }
    }

    /// Creates the farmer when the phone has none and leaves an existing
    /// one as it is. Safe to repeat.
    pub async fn execute(&self, phone: &Phone) -> Result<(), AppError> {
        self.farmers
            .create_if_absent(&Farmer::new(phone.clone(), Language::default()))
            .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::features::farmers::app::testing::{Call, Fakes, PHONE, phone};

    fn use_case(fakes: &Fakes) -> EnsureFarmerUseCase {
        EnsureFarmerUseCase::new(Arc::new(fakes.clone()))
    }

    #[tokio::test]
    async fn a_phone_without_a_farmer_gets_one() {
        let fakes = Fakes::new();

        use_case(&fakes).execute(&phone()).await.expect("ensured");

        assert!(fakes.has_farmer());
        assert_eq!(
            fakes.calls(),
            vec![Call::CreateFarmerIfAbsent {
                phone: PHONE.to_string()
            }],
            "the insert decides: no lookup comes before it"
        );
    }

    #[tokio::test]
    async fn a_farmer_who_is_already_there_is_left_as_they_are() {
        let fakes = Fakes::new().with_farmer();
        let before = fakes.farmer_created_at();

        use_case(&fakes).execute(&phone()).await.expect("ensured");

        assert_eq!(fakes.farmer_created_at(), before);
    }
}
