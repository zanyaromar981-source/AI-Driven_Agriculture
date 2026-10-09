use std::sync::Arc;

use crate::{
    app::{AppError as GlobalAppError, AuthContext},
    features::messages::{
        app::{AppError, MessageRepository, SenderDirectory},
        domain::StoredPhoto,
    },
};

pub struct ViewMyPhotoUseCase {
    repository: Arc<dyn MessageRepository>,
    farmers: Arc<dyn SenderDirectory>,
}

impl ViewMyPhotoUseCase {
    pub fn new(repository: Arc<dyn MessageRepository>, farmers: Arc<dyn SenderDirectory>) -> Self {
        Self {
            repository,
            farmers,
        }
    }

    /// Returns a photo of one of the signed-in farmer's own messages.
    /// Another farmer's photo is not found, exactly like a missing one.
    pub async fn execute(
        &self,
        auth_context: &AuthContext,
        message_id: i32,
        photo_id: i32,
    ) -> Result<StoredPhoto, AppError> {
        let Some(farmer_id) = self
            .farmers
            .farmer_id_of(auth_context.user().phone())
            .await?
        else {
            return Err(GlobalAppError::Unauthorized("Farmer not found".to_string()).into());
        };

        self.repository
            .find_photo(message_id, photo_id, Some(farmer_id))
            .await?
            .ok_or_else(|| GlobalAppError::NotFound.into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::features::messages::{
        app::testing::{
            Call, FARMER_ID, Fakes, a_message, a_message_of, a_stored_photo, auth_context,
        },
        domain::MessageState,
    };

    fn use_case(fakes: &Fakes) -> ViewMyPhotoUseCase {
        ViewMyPhotoUseCase::new(Arc::new(fakes.clone()), Arc::new(fakes.clone()))
    }

    #[tokio::test]
    async fn the_sender_gets_their_photo() {
        let fakes = Fakes::new()
            .with_stored(a_message(1))
            .with_photo(a_stored_photo(1));

        let photo = use_case(&fakes)
            .execute(&auth_context(), 1, 1)
            .await
            .expect("photo");

        assert_eq!(*photo.message_id(), 1);
        assert!(fakes.calls().contains(&Call::FindPhoto {
            message_id: 1,
            photo_id: 1,
            farmer_id: Some(FARMER_ID)
        }));
    }

    #[tokio::test]
    async fn another_farmers_photo_is_not_found() {
        let fakes = Fakes::new()
            .with_stored(a_message_of(1, FARMER_ID + 1, MessageState::New))
            .with_photo(a_stored_photo(1));

        let result = use_case(&fakes).execute(&auth_context(), 1, 1).await;

        assert!(
            matches!(
                result,
                Err(AppError::GlobalAppError(GlobalAppError::NotFound))
            ),
            "it must look exactly like a photo that does not exist"
        );
    }

    #[tokio::test]
    async fn a_photo_that_does_not_exist_is_not_found() {
        let fakes = Fakes::new().with_stored(a_message(1));

        let result = use_case(&fakes).execute(&auth_context(), 1, 2).await;

        assert!(matches!(
            result,
            Err(AppError::GlobalAppError(GlobalAppError::NotFound))
        ));
    }
}
