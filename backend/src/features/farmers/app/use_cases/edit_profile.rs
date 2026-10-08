use std::sync::Arc;

use crate::{
    app::{AppError as GlobalAppError, AuthContext},
    features::farmers::{
        app::{AppError, FarmerRepository},
        domain::{Farmer, FarmerName, Language},
    },
};

pub struct EditProfileInput {
    pub name: Option<FarmerName>,
    pub language: Language,
}

pub struct EditProfileUseCase {
    repository: Arc<dyn FarmerRepository>,
}

impl EditProfileUseCase {
    pub fn new(repository: Arc<dyn FarmerRepository>) -> Self {
        Self { repository }
    }

    pub async fn execute(
        &self,
        auth_context: &AuthContext,
        input: EditProfileInput,
    ) -> Result<Farmer, AppError> {
        let Some(mut farmer) = self
            .repository
            .find_by_phone(auth_context.user().phone())
            .await?
        else {
            return Err(GlobalAppError::NotFound.into());
        };

        farmer.update(input.name, input.language);

        let updated = self.repository.update(&farmer).await?;

        tracing::info!(
            farmer_id = updated.id().unwrap_or_default(),
            "profile edited"
        );

        Ok(updated)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::features::farmers::app::testing::{Call, Fakes, auth_context};

    fn input() -> EditProfileInput {
        EditProfileInput {
            name: Some(FarmerName::new("Hiwa K.".to_string()).expect("name")),
            language: Language::English,
        }
    }

    #[tokio::test]
    async fn edits_the_signed_in_farmers_own_profile() {
        let fakes = Fakes::new().with_farmer();
        let use_case = EditProfileUseCase::new(Arc::new(fakes.clone()));

        let farmer = use_case
            .execute(&auth_context(), input())
            .await
            .expect("farmer");

        assert_eq!(*farmer.language(), Language::English);
        assert!(fakes.calls().contains(&Call::UpdateFarmer));
    }

    #[tokio::test]
    async fn a_missing_farmer_is_not_written() {
        let fakes = Fakes::new();
        let use_case = EditProfileUseCase::new(Arc::new(fakes.clone()));

        let result = use_case.execute(&auth_context(), input()).await;

        assert!(matches!(
            result,
            Err(AppError::GlobalAppError(GlobalAppError::NotFound))
        ));
        assert!(!fakes.calls().contains(&Call::UpdateFarmer));
    }
}
