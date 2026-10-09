use std::sync::Arc;

use chrono::Utc;

use crate::{
    app::{AppError as GlobalAppError, StaffContext},
    features::messages::{
        app::{
            AppError, MessageFarmDirectory, MessageRecord, MessageRepository, SenderDirectory,
            describe,
        },
        domain::MessageState,
    },
};

pub struct SetMessageStateUseCase {
    repository: Arc<dyn MessageRepository>,
    farmers: Arc<dyn SenderDirectory>,
    farms: Arc<dyn MessageFarmDirectory>,
}

impl SetMessageStateUseCase {
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

    /// Moves a message to the state staff chose and returns it. Any state
    /// may follow any other, except that `replied` needs a reply. A reply
    /// already given stays with the message whatever state it moves to.
    pub async fn execute(
        &self,
        actor: &StaffContext,
        id: i32,
        state: MessageState,
    ) -> Result<MessageRecord, AppError> {
        // One statement decides: it writes only when the message is there
        // and the state is allowed for it.
        let Some(message) = self.repository.set_state(id, state, Utc::now()).await? else {
            // This read only finds out which of the two it was, for the
            // error. It decides nothing.
            let reason = match self.repository.find_by_id(id).await? {
                Some(message) => message.may_be_marked(state).err().map(AppError::from),
                None => None,
            };

            tracing::info!(
                staff_id = *actor.staff_id(),
                message_id = id,
                state = %String::from(state),
                "message state not changed"
            );

            return Err(reason.unwrap_or_else(|| GlobalAppError::NotFound.into()));
        };

        tracing::info!(
            staff_id = *actor.staff_id(),
            message_id = id,
            state = %String::from(state),
            "message state set by staff"
        );

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
    use crate::features::messages::{
        app::testing::{Call, FARMER_ID, Fakes, a_message, a_message_of, staff_context},
        domain::MessageError,
    };

    fn use_case(fakes: &Fakes) -> SetMessageStateUseCase {
        SetMessageStateUseCase::new(
            Arc::new(fakes.clone()),
            Arc::new(fakes.clone()),
            Arc::new(fakes.clone()),
        )
    }

    #[tokio::test]
    async fn marks_the_message_and_returns_it() {
        let fakes = Fakes::new().with_stored(a_message(1));

        let record = use_case(&fakes)
            .execute(&staff_context(), 1, MessageState::Read)
            .await
            .expect("record");

        assert_eq!(*record.message.state(), MessageState::Read);
        assert_eq!(
            fakes.calls()[0],
            Call::SetState {
                id: 1,
                state: MessageState::Read
            },
            "the write decides: there is no lookup before it"
        );
    }

    #[tokio::test]
    async fn a_closed_message_can_be_opened_again() {
        let fakes = Fakes::new().with_stored(a_message_of(1, FARMER_ID, MessageState::Closed));

        let record = use_case(&fakes)
            .execute(&staff_context(), 1, MessageState::New)
            .await
            .expect("record");

        assert_eq!(*record.message.state(), MessageState::New);
    }

    #[tokio::test]
    async fn a_message_that_does_not_exist_is_not_found() {
        let fakes = Fakes::new();

        let result = use_case(&fakes)
            .execute(&staff_context(), 1, MessageState::Read)
            .await;

        assert!(matches!(
            result,
            Err(AppError::GlobalAppError(GlobalAppError::NotFound))
        ));
    }

    #[tokio::test]
    async fn an_unanswered_message_cannot_be_marked_as_replied() {
        let fakes = Fakes::new().with_stored(a_message(1));

        let result = use_case(&fakes)
            .execute(&staff_context(), 1, MessageState::Replied)
            .await;

        assert!(matches!(
            result,
            Err(AppError::Message(MessageError::NoReply))
        ));
        assert_eq!(
            *fakes.stored()[0].state(),
            MessageState::New,
            "nothing was written"
        );
    }
}
