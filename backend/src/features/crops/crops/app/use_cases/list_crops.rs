use std::sync::Arc;

use crate::features::crops::{
    app::{AppError, CropRepository},
    domain::Crop,
};

pub struct ListCropsUseCase {
    repository: Arc<dyn CropRepository>,
}

impl ListCropsUseCase {
    pub fn new(repository: Arc<dyn CropRepository>) -> Self {
        Self { repository }
    }

    /// Returns the crops in their sort order: only the ones switched on for
    /// the app and the public page, or all of them for the editing screen.
    pub async fn execute(&self, only_active: bool) -> Result<Vec<Crop>, AppError> {
        let crops = self.repository.find_all(only_active).await?;

        tracing::debug!(only_active, returned = crops.len(), "crops listed");

        Ok(crops)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::features::crops::app::testing::{FakeCropRepository, RepositoryCall, a_crop};

    fn repository() -> FakeCropRepository {
        FakeCropRepository::holding(vec![
            a_crop("tomato", 30, true),
            a_crop("wheat", 10, true),
            a_crop("barley", 20, false),
        ])
    }

    fn codes(crops: &[Crop]) -> Vec<&str> {
        crops.iter().map(|crop| crop.code().as_str()).collect()
    }

    #[tokio::test]
    async fn the_public_list_holds_only_the_crops_switched_on_in_sort_order() {
        let repository = repository();
        let use_case = ListCropsUseCase::new(Arc::new(repository.clone()));

        let crops = use_case.execute(true).await.expect("crops");

        assert_eq!(codes(&crops), vec!["wheat", "tomato"]);
        assert_eq!(
            repository.calls(),
            vec![RepositoryCall::FindAll { only_active: true }]
        );
    }

    #[tokio::test]
    async fn the_editing_screen_also_gets_the_crops_switched_off() {
        let repository = repository();
        let use_case = ListCropsUseCase::new(Arc::new(repository.clone()));

        let crops = use_case.execute(false).await.expect("crops");

        assert_eq!(codes(&crops), vec!["wheat", "barley", "tomato"]);
        assert_eq!(
            repository.calls(),
            vec![RepositoryCall::FindAll { only_active: false }]
        );
    }

    #[tokio::test]
    async fn a_repository_failure_surfaces() {
        let use_case = ListCropsUseCase::new(Arc::new(FakeCropRepository::failing()));

        assert!(use_case.execute(true).await.is_err());
    }
}
