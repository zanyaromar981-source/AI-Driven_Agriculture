use std::sync::Arc;

use chrono::{DateTime, Utc};

use crate::{
    app::{AppError as GlobalAppError, StaffContext},
    features::crops::{
        app::{AppError, CropRepository},
        domain::{Crop, CropCode, CropDetails},
    },
};

pub struct UpdateCropUseCase {
    repository: Arc<dyn CropRepository>,
}

impl UpdateCropUseCase {
    pub fn new(repository: Arc<dyn CropRepository>) -> Self {
        Self { repository }
    }

    /// A staff member renames a crop, recolours it, reorders it or switches
    /// it on or off. The code names it and never changes.
    pub async fn execute(
        &self,
        actor: &StaffContext,
        code: CropCode,
        details: CropDetails,
        now: DateTime<Utc>,
    ) -> Result<Crop, AppError> {
        let Some(crop) = self.repository.update(&code, &details, now).await? else {
            tracing::info!(
                staff_id = *actor.staff_id(),
                crop = code.as_str(),
                "crop update refused: no such crop"
            );

            return Err(GlobalAppError::NotFound.into());
        };

        tracing::info!(
            staff_id = *actor.staff_id(),
            crop = code.as_str(),
            active = crop.details().active,
            "crop updated"
        );

        Ok(crop)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::features::crops::app::testing::{
        FakeCropRepository, RepositoryCall, a_crop, at, code, details, staff,
    };

    #[tokio::test]
    async fn replaces_everything_but_the_code_and_the_day_it_was_added() {
        let repository = FakeCropRepository::holding(vec![a_crop("wheat", 10, true)]);
        let use_case = UpdateCropUseCase::new(Arc::new(repository.clone()));

        let crop = use_case
            .execute(
                &staff(),
                code("wheat"),
                details("Bread wheat", 5, false),
                at(11),
            )
            .await
            .expect("crop");

        assert_eq!(crop.code().as_str(), "wheat");
        assert_eq!(crop.details(), &details("Bread wheat", 5, false));
        assert_eq!(*crop.created_at(), at(8), "the creation time is kept");
        assert_eq!(*crop.updated_at(), at(11));
        assert_eq!(
            repository.calls(),
            vec![RepositoryCall::Update {
                code: "wheat".to_string(),
                details: details("Bread wheat", 5, false),
            }],
            "the update decides; nothing is looked up first"
        );
    }

    #[tokio::test]
    async fn a_missing_crop_is_not_found_and_nothing_is_created() {
        let repository = FakeCropRepository::new();
        let use_case = UpdateCropUseCase::new(Arc::new(repository.clone()));

        let result = use_case
            .execute(&staff(), code("rice"), details("Rice", 5, true), at(11))
            .await;

        assert!(matches!(
            result,
            Err(AppError::GlobalAppError(GlobalAppError::NotFound))
        ));
        assert!(repository.stored("rice").is_none());
    }

    #[tokio::test]
    async fn a_repository_failure_surfaces() {
        let use_case = UpdateCropUseCase::new(Arc::new(FakeCropRepository::failing()));

        assert!(
            use_case
                .execute(&staff(), code("rice"), details("Rice", 5, true), at(11))
                .await
                .is_err()
        );
    }
}
