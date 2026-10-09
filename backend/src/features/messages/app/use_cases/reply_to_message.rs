use std::sync::Arc;

use chrono::Utc;

use crate::{
    app::{AppError as GlobalAppError, StaffContext},
    features::messages::{
        app::{
            AppError, MessageFarmDirectory, MessageRecord, MessageRepository, SenderDirectory,
            describe,
        },
        domain::{MessageText, Reply},
    },
};

pub struct ReplyToMessageInput {
    pub text_ku: MessageText,
    pub text_en: Option<MessageText>,
}

pub struct ReplyToMessageUseCase {
    repository: Arc<dyn MessageRepository>,
    farmers: Arc<dyn SenderDirectory>,
    farms: Arc<dyn MessageFarmDirectory>,
}

impl ReplyToMessageUseCase {
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

    /// Stores the reply the farmer will read and moves the message to
    /// `replied`. A second reply replaces the first. A message in any state
    /// can be replied to, a closed one too: the reply opens it again as
    /// `replied`, so the farmer is never shown an answer on a closed case.
    pub async fn execute(
        &self,
        actor: &StaffContext,
        id: i32,
        input: ReplyToMessageInput,
    ) -> Result<MessageRecord, AppError> {
        let reply = Reply::new(input.text_ku, input.text_en, *actor.staff_id(), Utc::now());

        let Some(message) = self.repository.reply(id, &reply).await? else {
            tracing::info!(
                staff_id = *actor.staff_id(),
                message_id = id,
                "reply refused: no such message"
            );

            return Err(GlobalAppError::NotFound.into());
        };

        tracing::info!(
            staff_id = *actor.staff_id(),
            message_id = id,
            "message replied to by staff"
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
        app::testing::{
            Call, FARMER_ID, Fakes, STAFF_ID, a_message, a_message_of, staff_context, text,
        },
        domain::MessageState,
    };

    fn use_case(fakes: &Fakes) -> ReplyToMessageUseCase {
        ReplyToMessageUseCase::new(
            Arc::new(fakes.clone()),
            Arc::new(fakes.clone()),
            Arc::new(fakes.clone()),
        )
    }

    fn input(text_ku: &str) -> ReplyToMessageInput {
        ReplyToMessageInput {
            text_ku: text(text_ku),
            text_en: None,
        }
    }

    #[tokio::test]
    async fn the_reply_is_kept_with_who_gave_it_and_the_message_becomes_replied() {
        let fakes = Fakes::new().with_stored(a_message(1));

        let record = use_case(&fakes)
            .execute(&staff_context(), 1, input("Water comes on Sunday"))
            .await
            .expect("record");

        let reply = record.message.reply().as_ref().expect("reply");

        assert_eq!(*record.message.state(), MessageState::Replied);
        assert_eq!(reply.text_ku().as_str(), "Water comes on Sunday");
        assert_eq!(*reply.replied_by(), STAFF_ID);
        assert_eq!(
            fakes.calls()[0],
            Call::Reply {
                id: 1,
                replied_by: STAFF_ID
            },
            "the write decides: there is no lookup before it"
        );
    }

    #[tokio::test]
    async fn a_second_reply_replaces_the_first() {
        let fakes = Fakes::new().with_stored(a_message(1));
        let use_case = use_case(&fakes);

        use_case
            .execute(&staff_context(), 1, input("First"))
            .await
            .expect("first");
        let record = use_case
            .execute(&staff_context(), 1, input("Second"))
            .await
            .expect("second");

        assert_eq!(
            record
                .message
                .reply()
                .as_ref()
                .expect("reply")
                .text_ku()
                .as_str(),
            "Second"
        );
    }

    #[tokio::test]
    async fn a_closed_message_can_be_replied_to_and_becomes_replied() {
        let fakes = Fakes::new().with_stored(a_message_of(1, FARMER_ID, MessageState::Closed));

        let record = use_case(&fakes)
            .execute(&staff_context(), 1, input("Answer"))
            .await
            .expect("record");

        assert_eq!(*record.message.state(), MessageState::Replied);
    }

    #[tokio::test]
    async fn a_message_that_does_not_exist_is_not_found() {
        let result = use_case(&Fakes::new())
            .execute(&staff_context(), 1, input("Answer"))
            .await;

        assert!(matches!(
            result,
            Err(AppError::GlobalAppError(GlobalAppError::NotFound))
        ));
    }
}
