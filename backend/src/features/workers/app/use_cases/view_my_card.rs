use std::sync::Arc;

use crate::{
    app::AuthContext,
    features::workers::{
        app::{AppError, WorkerRepository},
        domain::Worker,
    },
};

pub struct ViewMyCardUseCase {
    repository: Arc<dyn WorkerRepository>,
}

impl ViewMyCardUseCase {
    pub fn new(repository: Arc<dyn WorkerRepository>) -> Self {
        Self { repository }
    }

    /// The caller's own card, also when it is not available, or `None`
    /// when they have none.
    pub async fn execute(&self, auth_context: &AuthContext) -> Result<Option<Worker>, AppError> {
        let card = self
            .repository
            .find_by_phone(auth_context.user().phone())
            .await?;

        tracing::debug!(found = card.is_some(), "worker card viewed by its owner");

        Ok(card)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::features::workers::app::testing::{
        Call, Fakes, OTHER_PHONE, PHONE, a_worker, auth_context, unavailable,
    };

    #[tokio::test]
    async fn shows_the_callers_own_card_even_when_it_is_off_the_list() {
        let fakes = Fakes::new()
            .with_stored(a_worker(1, OTHER_PHONE))
            .with_stored(unavailable(&a_worker(2, PHONE)));

        let card = ViewMyCardUseCase::new(Arc::new(fakes.clone()))
            .execute(&auth_context())
            .await
            .expect("card")
            .expect("some");

        assert_eq!(*card.id(), Some(2));
        assert_eq!(
            fakes.calls(),
            vec![Call::FindByPhone {
                phone: PHONE.to_string()
            }]
        );
    }

    #[tokio::test]
    async fn someone_with_no_card_gets_none() {
        let fakes = Fakes::new().with_stored(a_worker(1, OTHER_PHONE));

        let card = ViewMyCardUseCase::new(Arc::new(fakes))
            .execute(&auth_context())
            .await
            .expect("card");

        assert_eq!(card, None);
    }
}
