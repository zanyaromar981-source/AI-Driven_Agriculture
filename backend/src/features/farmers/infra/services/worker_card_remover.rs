use std::sync::Arc;

use async_trait::async_trait;

use crate::{
    app::AppError as GlobalAppError,
    features::{
        farmers::app::{AppError, FarmerDataRemover},
        workers::app::WorkerRepository,
    },
    shared::Phone,
};

/// Removes a farmer's worker card on top of what another remover takes
/// away, by asking the workers feature through its own repository port. A
/// card shows a phone to every signed-in farmer, so it must not outlive the
/// account it belongs to.
#[derive(Debug)]
pub struct WorkersFeatureFarmerDataRemover {
    others: Arc<dyn FarmerDataRemover>,
    workers: Arc<dyn WorkerRepository>,
}

impl WorkersFeatureFarmerDataRemover {
    pub fn new(others: Arc<dyn FarmerDataRemover>, workers: Arc<dyn WorkerRepository>) -> Self {
        Self { others, workers }
    }
}

#[async_trait]
impl FarmerDataRemover for WorkersFeatureFarmerDataRemover {
    async fn remove_all_for(&self, phone: &Phone) -> Result<(), AppError> {
        // The card goes first: it is the part that shows a phone to others.
        let card_removed = self.workers.delete_by_phone(phone).await.map_err(|error| {
            tracing::error!(%error, "removing a farmer's worker card failed");

            AppError::from(GlobalAppError::InternalServerError)
        })?;

        tracing::info!(card_removed, "farmer's worker card removed");

        self.others.remove_all_for(phone).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::features::{
        farmers::app::testing::{Call, Fakes, PHONE, phone},
        workers::app::testing::{
            Call as WorkerCall, Fakes as WorkerFakes, OTHER_PHONE, PHONE as WORKER_PHONE, a_worker,
        },
    };

    #[tokio::test]
    async fn removes_the_farmers_card_and_then_everything_the_other_remover_does() {
        assert_eq!(PHONE, WORKER_PHONE, "both fakes speak of the same farmer");

        let others = Fakes::new();
        let workers = WorkerFakes::new()
            .with_stored(a_worker(1, WORKER_PHONE))
            .with_stored(a_worker(2, OTHER_PHONE));
        let remover = WorkersFeatureFarmerDataRemover::new(
            Arc::new(others.clone()),
            Arc::new(workers.clone()),
        );

        remover.remove_all_for(&phone()).await.expect("removed");

        assert_eq!(
            workers.stored(),
            vec![a_worker(2, OTHER_PHONE)],
            "another person's card stays"
        );
        assert_eq!(
            workers.calls(),
            vec![WorkerCall::DeleteByPhone {
                phone: PHONE.to_string()
            }]
        );
        assert_eq!(
            others.calls(),
            vec![Call::RemoveFarmerData {
                phone: PHONE.to_string()
            }]
        );
    }

    #[tokio::test]
    async fn a_failure_in_the_workers_feature_is_an_error_so_the_delete_is_sent_again() {
        let others = Fakes::new();
        let remover = WorkersFeatureFarmerDataRemover::new(
            Arc::new(others.clone()),
            Arc::new(WorkerFakes::new().failing()),
        );

        assert!(remover.remove_all_for(&phone()).await.is_err());
        assert!(
            others.calls().is_empty(),
            "the farmer is still there, so nothing else was removed either"
        );
    }
}
