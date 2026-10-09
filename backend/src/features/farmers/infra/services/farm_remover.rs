use std::sync::Arc;

use async_trait::async_trait;

use crate::{
    app::AppError as GlobalAppError,
    features::{
        farmers::app::{AppError, FarmRemover},
        farms::app::FarmRepository,
    },
    shared::Phone,
};

/// Removes a farmer's farms by asking the farms feature through its own
/// repository port, so the farms tables are touched only by their owner.
#[derive(Debug)]
pub struct FarmsFeatureFarmRemover {
    farms: Arc<dyn FarmRepository>,
}

impl FarmsFeatureFarmRemover {
    pub fn new(farms: Arc<dyn FarmRepository>) -> Self {
        Self { farms }
    }
}

#[async_trait]
impl FarmRemover for FarmsFeatureFarmRemover {
    async fn remove_all_for(&self, phone: &Phone) -> Result<u64, AppError> {
        self.farms
            .delete_all_by_owner(phone)
            .await
            .map_err(|error| {
                tracing::error!(%error, "removing a farmer's farms failed");

                GlobalAppError::InternalServerError.into()
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::features::farms::app::testing::{FakeFarmRepository, OWNER, RepositoryCall};

    fn phone() -> Phone {
        Phone::new(OWNER.to_string()).expect("phone")
    }

    #[tokio::test]
    async fn removes_only_the_farms_of_the_given_phone() {
        let farms = FakeFarmRepository::owning(2);
        let remover = FarmsFeatureFarmRemover::new(Arc::new(farms.clone()));

        assert_eq!(remover.remove_all_for(&phone()).await.expect("removed"), 2);
        assert_eq!(
            farms.calls(),
            vec![RepositoryCall::DeleteAllByOwner {
                owner: OWNER.to_string(),
            }]
        );
    }

    #[tokio::test]
    async fn a_failure_in_the_farms_feature_is_an_error() {
        let remover = FarmsFeatureFarmRemover::new(Arc::new(FakeFarmRepository::failing()));

        assert!(remover.remove_all_for(&phone()).await.is_err());
    }
}
