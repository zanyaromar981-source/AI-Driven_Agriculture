use std::sync::Arc;

use crate::{
    app::{AppError as GlobalAppError, AuthContext},
    features::farmers::{
        app::{AppError, FarmerRepository},
        domain::Farmer,
    },
};

pub struct ViewProfileUseCase {
    repository: Arc<dyn FarmerRepository>,
}

impl ViewProfileUseCase {
    pub fn new(repository: Arc<dyn FarmerRepository>) -> Self {
        Self { repository }
    }

    pub async fn execute(&self, auth_context: &AuthContext) -> Result<Farmer, AppError> {
        self.repository
            .find_by_phone(auth_context.user().phone())
            .await?
            .ok_or_else(|| GlobalAppError::NotFound.into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::features::farmers::app::testing::{Call, Fakes, PHONE, auth_context};

    #[tokio::test]
    async fn returns_the_signed_in_farmers_own_profile() {
        let fakes = Fakes::new().with_farmer();
        let use_case = ViewProfileUseCase::new(Arc::new(fakes.clone()));

        let farmer = use_case.execute(&auth_context()).await.expect("farmer");

        assert_eq!(farmer.phone().as_str(), PHONE);
        assert_eq!(
            fakes.calls(),
            vec![Call::FindFarmer {
                phone: PHONE.to_string()
            }]
        );
    }

    #[tokio::test]
    async fn is_not_found_for_a_token_whose_farmer_is_gone() {
        let use_case = ViewProfileUseCase::new(Arc::new(Fakes::new()));

        assert!(matches!(
            use_case.execute(&auth_context()).await,
            Err(AppError::GlobalAppError(GlobalAppError::NotFound))
        ));
    }
}
