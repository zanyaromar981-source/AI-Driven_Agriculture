use std::sync::Arc;

use crate::{
    app::{AppError as GlobalAppError, AuthContext, Pagination},
    features::messages::{
        app::{AppError, MessageRepository, SenderDirectory},
        domain::Message,
    },
};

pub struct ListMyMessagesUseCase {
    repository: Arc<dyn MessageRepository>,
    farmers: Arc<dyn SenderDirectory>,
}

impl ListMyMessagesUseCase {
    pub fn new(repository: Arc<dyn MessageRepository>, farmers: Arc<dyn SenderDirectory>) -> Self {
        Self {
            repository,
            farmers,
        }
    }

    /// Returns one page of the signed-in farmer's own messages, newest
    /// first, with how many they have in all.
    pub async fn execute(
        &self,
        auth_context: &AuthContext,
        pagination: Pagination,
    ) -> Result<(Vec<Message>, u64), AppError> {
        let Some(farmer_id) = self
            .farmers
            .farmer_id_of(auth_context.user().phone())
            .await?
        else {
            return Err(GlobalAppError::Unauthorized("Farmer not found".to_string()).into());
        };

        let (messages, count) = self
            .repository
            .find_page_by_farmer(farmer_id, &pagination)
            .await?;

        tracing::debug!(farmer_id, count, "farmer's messages listed");

        Ok((messages, count))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::features::messages::{
        app::testing::{Call, FARMER_ID, Fakes, PHONE, a_message, a_message_of, auth_context},
        domain::MessageState,
    };

    fn use_case(fakes: &Fakes) -> ListMyMessagesUseCase {
        ListMyMessagesUseCase::new(Arc::new(fakes.clone()), Arc::new(fakes.clone()))
    }

    #[tokio::test]
    async fn lists_only_the_messages_of_the_farmer_on_the_token() {
        let fakes = Fakes::new()
            .with_stored(a_message(1))
            .with_stored(a_message_of(2, FARMER_ID + 1, MessageState::New));

        let (messages, count) = use_case(&fakes)
            .execute(&auth_context(), Pagination::new(1, 20))
            .await
            .expect("messages");

        assert_eq!(count, 1);
        assert_eq!(*messages[0].id(), Some(1));
        assert_eq!(
            fakes.calls(),
            vec![
                Call::FarmerIdOf {
                    phone: PHONE.to_string()
                },
                Call::FindPageByFarmer {
                    farmer_id: FARMER_ID,
                    page: 1
                },
            ]
        );
    }

    #[tokio::test]
    async fn a_farmer_who_sent_nothing_gets_an_empty_list() {
        let (messages, count) = use_case(&Fakes::new())
            .execute(&auth_context(), Pagination::new(1, 20))
            .await
            .expect("messages");

        assert!(messages.is_empty());
        assert_eq!(count, 0);
    }

    #[tokio::test]
    async fn a_token_whose_farmer_is_gone_is_unauthorized() {
        let result = use_case(&Fakes::new().without_farmer())
            .execute(&auth_context(), Pagination::new(1, 20))
            .await;

        assert!(matches!(
            result,
            Err(AppError::GlobalAppError(GlobalAppError::Unauthorized(_)))
        ));
    }
}
