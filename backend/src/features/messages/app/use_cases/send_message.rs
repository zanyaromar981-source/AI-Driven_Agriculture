use std::sync::Arc;

use chrono::Utc;

use crate::{
    app::{AppError as GlobalAppError, AuthContext},
    features::messages::{
        app::{AppError, MessageFarmDirectory, MessageRepository, SendOutcome, SenderDirectory},
        domain::{
            IdempotencyKey, Message, MessageError, MessageKind, MessageText, Photo, SendingLimit,
        },
    },
};

pub struct SendMessageInput {
    pub kind: MessageKind,
    pub text: MessageText,
    pub farm_id: Option<i32>,
    pub photos: Vec<Photo>,
    pub idempotency_key: Option<IdempotencyKey>,
}

pub struct SendMessageUseCase {
    repository: Arc<dyn MessageRepository>,
    farmers: Arc<dyn SenderDirectory>,
    farms: Arc<dyn MessageFarmDirectory>,
}

impl SendMessageUseCase {
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

    /// Stores the farmer's message with its photos and returns it. A send
    /// that repeats the idempotency key of an earlier one returns that
    /// earlier message and stores nothing.
    pub async fn execute(
        &self,
        auth_context: &AuthContext,
        input: SendMessageInput,
    ) -> Result<Message, AppError> {
        let phone = auth_context.user().phone();

        let Some(farmer_id) = self.farmers.farmer_id_of(phone).await? else {
            // The farmer was removed after the token was checked.
            return Err(GlobalAppError::Unauthorized("Farmer not found".to_string()).into());
        };

        // A repeat is answered before anything else is checked: the farm it
        // named may be gone by now, and the first send already succeeded.
        if let Some(key) = &input.idempotency_key
            && let Some(existing) = self
                .repository
                .find_by_idempotency_key(farmer_id, key)
                .await?
        {
            tracing::info!(
                message_id = existing.id().unwrap_or_default(),
                farmer_id,
                "send repeated: returning the message already stored"
            );

            return Ok(existing);
        }

        if let Some(farm_id) = input.farm_id
            && !self.farms.is_owned_by(farm_id, phone).await?
        {
            tracing::info!(
                farmer_id,
                farm_id,
                "send refused: not one of the farmer's farms"
            );

            return Err(GlobalAppError::NotFound.into());
        }

        let now = Utc::now();

        let message = Message::new(
            farmer_id,
            input.farm_id,
            input.kind,
            input.text,
            input.photos,
            input.idempotency_key,
            now,
        )?;

        // The repository counts and writes under one lock per farmer, so
        // the limit holds for sends that arrive together.
        let outcome = self
            .repository
            .send(
                &message,
                SendingLimit::MAX_MESSAGES,
                SendingLimit::window_start(now),
            )
            .await?;

        match outcome {
            SendOutcome::Stored(stored) => {
                tracing::info!(
                    message_id = stored.id().unwrap_or_default(),
                    farmer_id,
                    kind = %String::from(*stored.kind()),
                    photos = stored.photos().len(),
                    "message stored"
                );

                Ok(stored)
            }
            SendOutcome::Repeated(existing) => {
                tracing::info!(
                    message_id = existing.id().unwrap_or_default(),
                    farmer_id,
                    "send raced its own repeat: returning the message already stored"
                );

                Ok(existing)
            }
            SendOutcome::LimitReached { oldest_counted } => {
                let retry_after_s = SendingLimit::retry_after_s(oldest_counted, now);

                tracing::info!(
                    farmer_id,
                    max = SendingLimit::MAX_MESSAGES,
                    retry_after_s,
                    "send refused: message limit reached"
                );

                Err(MessageError::TooManyMessages {
                    max: SendingLimit::MAX_MESSAGES,
                    retry_after_s,
                }
                .into())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::features::messages::app::testing::{
        Call, FARM_ID, FARMER_ID, Fakes, PHONE, a_key, a_message, a_photo, auth_context, text,
    };

    fn use_case(fakes: &Fakes) -> SendMessageUseCase {
        SendMessageUseCase::new(
            Arc::new(fakes.clone()),
            Arc::new(fakes.clone()),
            Arc::new(fakes.clone()),
        )
    }

    fn input() -> SendMessageInput {
        SendMessageInput {
            kind: MessageKind::Question,
            text: text("When does the water come?"),
            farm_id: None,
            photos: vec![],
            idempotency_key: None,
        }
    }

    #[tokio::test]
    async fn stores_the_message_for_the_farmer_on_the_token() {
        let fakes = Fakes::new();

        let message = use_case(&fakes)
            .execute(&auth_context(), input())
            .await
            .expect("message");

        assert_eq!(*message.farmer_id(), FARMER_ID);
        assert_eq!(*message.id(), Some(1));
        assert_eq!(
            fakes.calls(),
            vec![
                Call::FarmerIdOf {
                    phone: PHONE.to_string()
                },
                Call::Send {
                    farmer_id: FARMER_ID,
                    photos: 0,
                    max_messages: 20,
                },
            ],
            "with no key and no farm there is nothing else to ask"
        );
    }

    #[tokio::test]
    async fn the_photos_travel_to_the_store_with_the_message() {
        let fakes = Fakes::new();

        let message = use_case(&fakes)
            .execute(
                &auth_context(),
                SendMessageInput {
                    photos: vec![a_photo(), a_photo()],
                    ..input()
                },
            )
            .await
            .expect("message");

        assert_eq!(message.photos().len(), 2);
        assert!(fakes.calls().contains(&Call::Send {
            farmer_id: FARMER_ID,
            photos: 2,
            max_messages: 20,
        }));
    }

    #[tokio::test]
    async fn a_farm_is_checked_against_the_phone_on_the_token() {
        let fakes = Fakes::new();

        let message = use_case(&fakes)
            .execute(
                &auth_context(),
                SendMessageInput {
                    farm_id: Some(FARM_ID),
                    ..input()
                },
            )
            .await
            .expect("message");

        assert_eq!(*message.farm_id(), Some(FARM_ID));
        assert!(fakes.calls().contains(&Call::IsOwnedBy {
            farm_id: FARM_ID,
            phone: PHONE.to_string()
        }));
    }

    #[tokio::test]
    async fn another_farmers_farm_is_not_found_and_nothing_is_stored() {
        let fakes = Fakes::new().owning_no_farm();

        let result = use_case(&fakes)
            .execute(
                &auth_context(),
                SendMessageInput {
                    farm_id: Some(FARM_ID),
                    ..input()
                },
            )
            .await;

        assert!(matches!(
            result,
            Err(AppError::GlobalAppError(GlobalAppError::NotFound))
        ));
        assert!(
            !fakes
                .calls()
                .iter()
                .any(|call| matches!(call, Call::Send { .. })),
            "a refused send must not write"
        );
    }

    #[tokio::test]
    async fn too_many_photos_are_refused_before_anything_is_stored() {
        let fakes = Fakes::new();

        let result = use_case(&fakes)
            .execute(
                &auth_context(),
                SendMessageInput {
                    photos: vec![a_photo(); 5],
                    ..input()
                },
            )
            .await;

        assert!(matches!(
            result,
            Err(AppError::Message(MessageError::TooManyPhotos(4)))
        ));
        assert!(fakes.stored().is_empty());
    }

    #[tokio::test]
    async fn a_repeated_send_returns_the_message_already_stored() {
        let fakes = Fakes::new().with_stored(a_message(5)).owning_no_farm();

        let message = use_case(&fakes)
            .execute(
                &auth_context(),
                SendMessageInput {
                    idempotency_key: Some(a_key()),
                    farm_id: Some(FARM_ID),
                    ..input()
                },
            )
            .await
            .expect("message");

        assert_eq!(*message.id(), Some(5));
        assert_eq!(
            fakes.calls(),
            vec![
                Call::FarmerIdOf {
                    phone: PHONE.to_string()
                },
                Call::FindByIdempotencyKey {
                    farmer_id: FARMER_ID,
                    key: "send-1".to_string()
                },
            ],
            "a repeat must not check the farm again, count against the limit or write"
        );
    }

    #[tokio::test]
    async fn a_send_that_races_its_own_repeat_gets_the_message_that_won() {
        let fakes = Fakes::new().losing_the_race_to_send(a_message(5));

        let message = use_case(&fakes)
            .execute(
                &auth_context(),
                SendMessageInput {
                    idempotency_key: Some(a_key()),
                    ..input()
                },
            )
            .await
            .expect("message");

        assert_eq!(*message.id(), Some(5));
        assert_eq!(fakes.stored().len(), 1, "only the winner is stored");
    }

    #[tokio::test]
    async fn a_farmer_at_the_limit_is_told_how_long_to_wait() {
        let oldest = Utc::now() - chrono::Duration::hours(23);
        let fakes = Fakes::new().at_the_limit_since(oldest);

        let result = use_case(&fakes).execute(&auth_context(), input()).await;

        let Err(AppError::Message(MessageError::TooManyMessages { max, retry_after_s })) = result
        else {
            panic!("expected the limit, got {result:?}");
        };

        assert_eq!(max, 20);
        assert!(
            (3_590..=3_600).contains(&retry_after_s),
            "the oldest message leaves the window in an hour, got {retry_after_s}"
        );
        assert!(fakes.stored().is_empty());
    }

    #[tokio::test]
    async fn a_token_whose_farmer_is_gone_is_unauthorized() {
        let fakes = Fakes::new().without_farmer();

        let result = use_case(&fakes).execute(&auth_context(), input()).await;

        assert!(matches!(
            result,
            Err(AppError::GlobalAppError(GlobalAppError::Unauthorized(_)))
        ));
        assert_eq!(fakes.calls().len(), 1);
    }

    #[tokio::test]
    async fn a_repository_failure_surfaces_instead_of_storing() {
        let fakes = Fakes::new().failing();

        assert!(
            use_case(&fakes)
                .execute(&auth_context(), input())
                .await
                .is_err()
        );
        assert!(fakes.stored().is_empty());
    }
}
