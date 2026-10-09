use std::sync::Arc;

use chrono::Utc;

use crate::{
    app::AuthContext,
    features::workers::{
        app::{AppError, WorkerRepository},
        domain::{Worker, WorkerDetails},
    },
};

pub struct PutMyCardUseCase {
    repository: Arc<dyn WorkerRepository>,
}

impl PutMyCardUseCase {
    pub fn new(repository: Arc<dyn WorkerRepository>) -> Self {
        Self { repository }
    }

    /// Puts up the caller's card, or replaces the one they have. The phone
    /// on the card is the phone they signed in with, so nobody can list
    /// another person's number. Sending the same card again changes nothing
    /// but the time of the update.
    pub async fn execute(
        &self,
        auth_context: &AuthContext,
        details: WorkerDetails,
    ) -> Result<Worker, AppError> {
        let card = Worker::new(auth_context.user().phone().clone(), details, Utc::now());

        let saved = self.repository.save(&card).await?;

        tracing::info!(
            worker_id = *saved.id(),
            available = *saved.available(),
            "worker card saved"
        );

        Ok(saved)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::features::workers::app::testing::{
        Call, Fakes, OTHER_PHONE, PHONE, a_worker, auth_context, details,
    };

    #[tokio::test]
    async fn the_card_is_stored_under_the_signed_in_phone() {
        let fakes = Fakes::new();

        let saved = PutMyCardUseCase::new(Arc::new(fakes.clone()))
            .execute(&auth_context(), details("Azad", 25_000))
            .await
            .expect("saved");

        assert_eq!(saved.phone().as_str(), PHONE);
        assert_eq!(saved.name().as_str(), "Azad");
        assert_eq!(
            fakes.calls(),
            vec![Call::Save {
                phone: PHONE.to_string()
            }]
        );
    }

    #[tokio::test]
    async fn a_second_put_replaces_the_first_and_leaves_one_card() {
        let fakes = Fakes::new().with_stored(a_worker(1, OTHER_PHONE));
        let use_case = PutMyCardUseCase::new(Arc::new(fakes.clone()));

        use_case
            .execute(&auth_context(), details("Azad", 25_000))
            .await
            .expect("first");
        let second = use_case
            .execute(&auth_context(), details("Azad Karim", 30_000))
            .await
            .expect("second");

        assert_eq!(second.cost().value(), 30_000);
        assert_eq!(
            fakes.stored().len(),
            2,
            "one card each for the two phones, never a second for the same phone"
        );
    }

    #[tokio::test]
    async fn a_repository_failure_surfaces() {
        let result = PutMyCardUseCase::new(Arc::new(Fakes::new().failing()))
            .execute(&auth_context(), details("Azad", 25_000))
            .await;

        assert!(result.is_err());
    }
}
