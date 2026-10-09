use std::sync::Arc;

use crate::{
    app::AppError as GlobalAppError,
    features::messages::app::{
        AppError, MessageFarmDirectory, MessageRecord, MessageRepository, SenderDirectory, describe,
    },
};

pub struct ViewMessageUseCase {
    repository: Arc<dyn MessageRepository>,
    farmers: Arc<dyn SenderDirectory>,
    farms: Arc<dyn MessageFarmDirectory>,
}

impl ViewMessageUseCase {
    pub fn new(
        repository: Arc<dyn MessageRepository>,
        farmers: Arc<dyn SenderDirectory>,
        farms: Arc<dyn MessageFarmDirectory>,
    ) -> Self {
        Self {
            repository,
            farmers,
            farms,
        }
    }

    /// Returns one message of any farmer, with its farmer and farm.
    pub async fn execute(&self, id: i32) -> Result<MessageRecord, AppError> {
        let message = self
            .repository
            .find_by_id(id)
            .await?
            .ok_or(GlobalAppError::NotFound)?;

        describe(self.farmers.as_ref(), self.farms.as_ref(), vec![message])
            .await?
            .into_iter()
            .next()
            .ok_or_else(|| GlobalAppError::NotFound.into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::features::messages::app::testing::{Call, Fakes, a_card, a_contact, a_message};

    fn use_case(fakes: &Fakes) -> ViewMessageUseCase {
        ViewMessageUseCase::new(
            Arc::new(fakes.clone()),
            Arc::new(fakes.clone()),
            Arc::new(fakes.clone()),
        )
    }

    #[tokio::test]
    async fn returns_the_message_with_its_farmer_and_farm() {
        let fakes = Fakes::new()
            .with_stored(a_message(4))
            .with_contact(a_contact())
            .with_card(a_card());

        let record = use_case(&fakes).execute(4).await.expect("record");

        assert_eq!(*record.message.id(), Some(4));
        assert_eq!(record.farmer, Some(a_contact()));
        assert_eq!(record.farm, Some(a_card()));
        assert_eq!(fakes.calls()[0], Call::FindById { id: 4 });
    }

    #[tokio::test]
    async fn a_message_that_does_not_exist_is_not_found() {
        let fakes = Fakes::new();

        let result = use_case(&fakes).execute(4).await;

        assert!(matches!(
            result,
            Err(AppError::GlobalAppError(GlobalAppError::NotFound))
        ));
        assert_eq!(fakes.calls(), vec![Call::FindById { id: 4 }]);
    }
}
