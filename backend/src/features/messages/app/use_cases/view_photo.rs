use std::sync::Arc;

use crate::{
    app::AppError as GlobalAppError,
    features::messages::{
        app::{AppError, MessageRepository},
        domain::StoredPhoto,
    },
};

pub struct ViewPhotoUseCase {
    repository: Arc<dyn MessageRepository>,
}

impl ViewPhotoUseCase {
    pub fn new(repository: Arc<dyn MessageRepository>) -> Self {
        Self { repository }
    }

    /// Returns a photo of any farmer's message, for Ministry staff.
    pub async fn execute(&self, message_id: i32, photo_id: i32) -> Result<StoredPhoto, AppError> {
        self.repository
            .find_photo(message_id, photo_id, None)
            .await?
            .ok_or_else(|| GlobalAppError::NotFound.into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::features::messages::app::testing::{Call, Fakes, a_message, a_stored_photo};

    #[tokio::test]
    async fn returns_the_photo_of_that_message() {
        let fakes = Fakes::new()
            .with_stored(a_message(1))
            .with_photo(a_stored_photo(1));

        let photo = ViewPhotoUseCase::new(Arc::new(fakes.clone()))
            .execute(1, 1)
            .await
            .expect("photo");

        assert_eq!(*photo.id(), 1);
        assert_eq!(
            fakes.calls(),
            vec![Call::FindPhoto {
                message_id: 1,
                photo_id: 1,
                farmer_id: None
            }]
        );
    }

    #[tokio::test]
    async fn a_photo_asked_for_under_another_message_is_not_found() {
        let fakes = Fakes::new()
            .with_stored(a_message(1))
            .with_photo(a_stored_photo(1));

        let result = ViewPhotoUseCase::new(Arc::new(fakes)).execute(2, 1).await;

        assert!(matches!(
            result,
            Err(AppError::GlobalAppError(GlobalAppError::NotFound))
        ));
    }
}
