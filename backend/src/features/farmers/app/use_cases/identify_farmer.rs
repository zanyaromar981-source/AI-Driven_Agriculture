use std::sync::Arc;

use crate::{
    app::AppError as GlobalAppError,
    features::farmers::{
        app::{AppError, FarmerRepository},
        domain::Farmer,
    },
    shared::Phone,
};

/// Answers, on every request with a farmer's token, whether the farmer the
/// token names is still there.
pub struct IdentifyFarmerUseCase {
    farmers: Arc<dyn FarmerRepository>,
}

impl IdentifyFarmerUseCase {
    pub fn new(farmers: Arc<dyn FarmerRepository>) -> Self {
        Self { farmers }
    }

    /// A token outlives a farmer staff have removed, so the farmer is read
    /// again each time: without one the token is useless at once.
    pub async fn execute(&self, phone: &Phone) -> Result<Farmer, AppError> {
        let Some(farmer) = self.farmers.find_by_phone(phone).await? else {
            tracing::info!("farmer token refused: the farmer is gone");

            return Err(GlobalAppError::Unauthorized("The farmer is gone".to_string()).into());
        };

        Ok(farmer)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        app::{ErrorKind, ToErrorInfo},
        features::farmers::app::testing::{Call, Fakes, PHONE, phone},
    };

    fn use_case(fakes: &Fakes) -> IdentifyFarmerUseCase {
        IdentifyFarmerUseCase::new(Arc::new(fakes.clone()))
    }

    #[tokio::test]
    async fn a_token_whose_farmer_exists_is_accepted_after_one_lookup_by_its_phone() {
        let fakes = Fakes::new().with_farmer();

        let farmer = use_case(&fakes).execute(&phone()).await.expect("farmer");

        assert_eq!(farmer.phone().as_str(), PHONE);
        assert_eq!(
            fakes.calls(),
            vec![Call::FindFarmer {
                phone: PHONE.to_string()
            }]
        );
    }

    #[tokio::test]
    async fn a_token_whose_farmer_was_removed_is_unauthorized() {
        let fakes = Fakes::new();

        let error = use_case(&fakes)
            .execute(&phone())
            .await
            .expect_err("refused");

        assert!(matches!(
            error,
            AppError::GlobalAppError(GlobalAppError::Unauthorized(_))
        ));
        assert_eq!(error.to_error_info().code, "unauthorized");
    }

    #[tokio::test]
    async fn a_database_failure_is_a_server_fault_not_a_refusal() {
        let fakes = Fakes::new().with_farmer().failing_to_read_farmers();

        let error = use_case(&fakes)
            .execute(&phone())
            .await
            .expect_err("failed");

        assert_eq!(
            error.to_error_info().kind,
            ErrorKind::Persistence,
            "the app signs the farmer out on 401, so an outage must not look like one"
        );
    }
}
