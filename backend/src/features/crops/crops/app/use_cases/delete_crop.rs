use std::sync::Arc;

use crate::{
    app::StaffContext,
    features::crops::{
        app::{AppError, CropRepository, CropUsage},
        domain::{CropCode, CropError},
    },
};

pub struct DeleteCropUseCase {
    repository: Arc<dyn CropRepository>,
    usages: Vec<Arc<dyn CropUsage>>,
}

impl DeleteCropUseCase {
    /// `usages` holds one entry for each feature that stores crop codes.
    pub fn new(repository: Arc<dyn CropRepository>, usages: Vec<Arc<dyn CropUsage>>) -> Self {
        Self { repository, usages }
    }

    /// A staff member removes a crop nothing uses. Removing one that is
    /// already gone succeeds: gone is what was asked.
    ///
    /// The features that store crop codes share no key with this table, so
    /// "nothing uses it" is asked, not enforced: a farm or a listing saved
    /// with the crop between the last question and the delete is not seen.
    /// The questions are therefore the last thing before the delete. Such a
    /// farm keeps its crop code and reads back as before; only the name and
    /// colour are then missing from the crop list until the crop is added
    /// again.
    pub async fn execute(&self, actor: &StaffContext, code: CropCode) -> Result<(), AppError> {
        for usage in &self.usages {
            if usage.is_used(&code).await? {
                tracing::info!(
                    staff_id = *actor.staff_id(),
                    crop = code.as_str(),
                    "crop delete refused: the crop is in use"
                );

                return Err(CropError::InUse.into());
            }
        }

        let removed = self.repository.delete(&code).await?;

        tracing::info!(
            staff_id = *actor.staff_id(),
            crop = code.as_str(),
            removed,
            "crop deleted"
        );

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::features::crops::app::testing::{
        FakeCropRepository, FakeCropUsage, RepositoryCall, a_crop, code, staff,
    };

    fn use_case(
        repository: &FakeCropRepository,
        farms: &FakeCropUsage,
        alwa: &FakeCropUsage,
    ) -> DeleteCropUseCase {
        DeleteCropUseCase::new(
            Arc::new(repository.clone()),
            vec![Arc::new(farms.clone()), Arc::new(alwa.clone())],
        )
    }

    #[tokio::test]
    async fn removes_a_crop_nothing_uses_after_asking_every_feature() {
        let repository = FakeCropRepository::holding(vec![a_crop("rice", 10, true)]);
        let (farms, alwa) = (FakeCropUsage::default(), FakeCropUsage::default());

        use_case(&repository, &farms, &alwa)
            .execute(&staff(), code("rice"))
            .await
            .expect("delete");

        assert!(repository.stored("rice").is_none());
        assert_eq!(farms.asked(), vec!["rice"]);
        assert_eq!(alwa.asked(), vec!["rice"]);
        assert_eq!(
            repository.calls(),
            vec![RepositoryCall::Delete {
                code: "rice".to_string()
            }]
        );
    }

    #[tokio::test]
    async fn a_crop_a_farm_uses_is_kept() {
        let repository = FakeCropRepository::holding(vec![a_crop("rice", 10, true)]);
        let (farms, alwa) = (FakeCropUsage::using(&["rice"]), FakeCropUsage::default());

        let result = use_case(&repository, &farms, &alwa)
            .execute(&staff(), code("rice"))
            .await;

        assert!(matches!(result, Err(AppError::Crop(CropError::InUse))));
        assert!(repository.stored("rice").is_some());
        assert!(repository.calls().is_empty(), "nothing may be written");
    }

    #[tokio::test]
    async fn a_crop_a_listing_or_a_price_uses_is_kept() {
        let repository = FakeCropRepository::holding(vec![a_crop("rice", 10, true)]);
        let (farms, alwa) = (FakeCropUsage::default(), FakeCropUsage::using(&["rice"]));

        let result = use_case(&repository, &farms, &alwa)
            .execute(&staff(), code("rice"))
            .await;

        assert!(matches!(result, Err(AppError::Crop(CropError::InUse))));
        assert!(repository.calls().is_empty(), "nothing may be written");
    }

    #[tokio::test]
    async fn a_crop_switched_off_but_still_in_use_is_kept_too() {
        let repository = FakeCropRepository::holding(vec![a_crop("rice", 10, false)]);
        let (farms, alwa) = (FakeCropUsage::using(&["rice"]), FakeCropUsage::default());

        let result = use_case(&repository, &farms, &alwa)
            .execute(&staff(), code("rice"))
            .await;

        assert!(matches!(result, Err(AppError::Crop(CropError::InUse))));
    }

    #[tokio::test]
    async fn removing_a_crop_that_is_already_gone_succeeds() {
        let repository = FakeCropRepository::new();
        let (farms, alwa) = (FakeCropUsage::default(), FakeCropUsage::default());
        let use_case = use_case(&repository, &farms, &alwa);

        use_case
            .execute(&staff(), code("rice"))
            .await
            .expect("first");
        use_case
            .execute(&staff(), code("rice"))
            .await
            .expect("again");
    }

    #[tokio::test]
    async fn when_a_feature_cannot_answer_nothing_is_deleted() {
        let repository = FakeCropRepository::holding(vec![a_crop("rice", 10, true)]);
        let (farms, alwa) = (FakeCropUsage::default(), FakeCropUsage::failing());

        let result = use_case(&repository, &farms, &alwa)
            .execute(&staff(), code("rice"))
            .await;

        assert!(result.is_err());
        assert!(
            repository.stored("rice").is_some(),
            "not knowing whether it is used is not the same as unused"
        );
    }
}
