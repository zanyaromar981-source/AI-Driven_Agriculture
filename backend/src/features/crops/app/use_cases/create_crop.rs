use std::sync::Arc;

use chrono::{DateTime, Utc};

use crate::{
    app::StaffContext,
    features::crops::{
        app::{AppError, CropRepository},
        domain::{Crop, CropCode, CropDetails, CropError},
    },
};

pub struct CreateCropInput {
    pub code: CropCode,
    pub details: CropDetails,
}

pub struct CreateCropUseCase {
    repository: Arc<dyn CropRepository>,
}

impl CreateCropUseCase {
    pub fn new(repository: Arc<dyn CropRepository>) -> Self {
        Self { repository }
    }

    /// A staff member adds a crop. A code that is taken is refused, and the
    /// crop that has it is left as it was.
    pub async fn execute(
        &self,
        actor: &StaffContext,
        input: CreateCropInput,
        now: DateTime<Utc>,
    ) -> Result<Crop, AppError> {
        let crop = Crop::new(input.code, input.details, now);

        let Some(created) = self.repository.create(&crop).await? else {
            tracing::info!(
                staff_id = *actor.staff_id(),
                crop = crop.code().as_str(),
                "crop refused: the code is taken"
            );

            return Err(CropError::AlreadyExists.into());
        };

        tracing::info!(
            staff_id = *actor.staff_id(),
            crop = created.code().as_str(),
            "crop created"
        );

        Ok(created)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::features::crops::app::testing::{
        FakeCropRepository, RepositoryCall, a_crop, at, code, details, staff,
    };

    fn input(code_value: &str) -> CreateCropInput {
        CreateCropInput {
            code: code(code_value),
            details: details("Rice", 170, true),
        }
    }

    #[tokio::test]
    async fn adds_a_crop_under_a_new_code() {
        let repository = FakeCropRepository::holding(vec![a_crop("wheat", 10, true)]);
        let use_case = CreateCropUseCase::new(Arc::new(repository.clone()));

        let crop = use_case
            .execute(&staff(), input("rice"), at(9))
            .await
            .expect("crop");

        assert_eq!(crop.code().as_str(), "rice");
        assert_eq!(*crop.created_at(), at(9));
        assert!(repository.stored("rice").is_some());
        assert_eq!(
            repository.calls(),
            vec![RepositoryCall::Create {
                code: "rice".to_string()
            }],
            "the insert decides; nothing is looked up first"
        );
    }

    #[tokio::test]
    async fn a_taken_code_is_refused_and_the_crop_that_has_it_is_untouched() {
        let repository = FakeCropRepository::holding(vec![a_crop("rice", 10, false)]);
        let use_case = CreateCropUseCase::new(Arc::new(repository.clone()));

        let result = use_case.execute(&staff(), input("rice"), at(9)).await;

        assert!(matches!(
            result,
            Err(AppError::Crop(CropError::AlreadyExists))
        ));
        assert_eq!(
            repository.stored("rice").expect("stored"),
            a_crop("rice", 10, false)
        );
    }

    #[tokio::test]
    async fn a_repository_failure_surfaces() {
        let use_case = CreateCropUseCase::new(Arc::new(FakeCropRepository::failing()));

        assert!(
            use_case
                .execute(&staff(), input("rice"), at(9))
                .await
                .is_err()
        );
    }
}
