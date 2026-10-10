use std::sync::Arc;

use crate::features::crops::{
    app::{AppError, CropRepository},
    domain::Crop,
};

/// Which of the products a reader is after.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CropList {
    /// What a farm can be painted with: the crops switched on.
    FieldCrops,
    /// What can be sold: every product switched on, crops first.
    Products,
    /// The editing screen: every product, switched on or not.
    Everything,
}

pub struct ListCropsUseCase {
    repository: Arc<dyn CropRepository>,
}

impl ListCropsUseCase {
    pub fn new(repository: Arc<dyn CropRepository>) -> Self {
        Self { repository }
    }

    /// Returns the products asked for. Crops alone and the editing screen
    /// come in their sort order; the products for sale come group by group,
    /// crops first.
    pub async fn execute(&self, list: CropList) -> Result<Vec<Crop>, AppError> {
        let only_active = list != CropList::Everything;
        let mut crops = self.repository.find_all(only_active).await?;

        match list {
            CropList::FieldCrops => crops.retain(Crop::is_field_crop),
            CropList::Products => {
                crops.sort_by(|one, other| one.display_key().cmp(&other.display_key()))
            }
            CropList::Everything => {}
        }

        tracing::debug!(?list, returned = crops.len(), "crops listed");

        Ok(crops)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::features::crops::{
        app::testing::{FakeCropRepository, RepositoryCall, a_crop, a_product},
        domain::{ProductGroup, ProductUnit},
    };

    fn repository() -> FakeCropRepository {
        FakeCropRepository::holding(vec![
            a_crop("tomato", 30, true),
            a_crop("wheat", 10, true),
            a_crop("barley", 20, false),
            // Sorted before every crop, to show that the group still leads.
            a_product("eggs", 5, ProductGroup::FishMeatEggs, ProductUnit::Tray30),
            a_product("cow", 1, ProductGroup::Animals, ProductUnit::Head),
        ])
    }

    fn codes(crops: &[Crop]) -> Vec<&str> {
        crops.iter().map(|crop| crop.code().as_str()).collect()
    }

    #[tokio::test]
    async fn the_public_list_holds_only_the_crops_switched_on_in_sort_order() {
        let repository = repository();
        let use_case = ListCropsUseCase::new(Arc::new(repository.clone()));

        let crops = use_case.execute(CropList::FieldCrops).await.expect("crops");

        assert_eq!(codes(&crops), vec!["wheat", "tomato"]);
        assert_eq!(
            repository.calls(),
            vec![RepositoryCall::FindAll { only_active: true }]
        );
    }

    #[tokio::test]
    async fn the_products_for_sale_come_crops_first_then_group_by_group() {
        let repository = repository();
        let use_case = ListCropsUseCase::new(Arc::new(repository.clone()));

        let crops = use_case.execute(CropList::Products).await.expect("crops");

        assert_eq!(
            codes(&crops),
            vec!["wheat", "tomato", "eggs", "cow"],
            "a low sort order does not lift a product above the crops"
        );
        assert_eq!(
            repository.calls(),
            vec![RepositoryCall::FindAll { only_active: true }]
        );
    }

    #[tokio::test]
    async fn the_editing_screen_gets_every_product_also_the_ones_switched_off() {
        let repository = repository();
        let use_case = ListCropsUseCase::new(Arc::new(repository.clone()));

        let crops = use_case.execute(CropList::Everything).await.expect("crops");

        assert_eq!(
            codes(&crops),
            vec!["cow", "eggs", "wheat", "barley", "tomato"]
        );
        assert_eq!(
            repository.calls(),
            vec![RepositoryCall::FindAll { only_active: false }]
        );
    }

    #[tokio::test]
    async fn a_repository_failure_surfaces() {
        let use_case = ListCropsUseCase::new(Arc::new(FakeCropRepository::failing()));

        assert!(use_case.execute(CropList::FieldCrops).await.is_err());
    }
}
