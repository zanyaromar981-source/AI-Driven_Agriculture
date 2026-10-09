use std::sync::Arc;

use crate::{
    app::AppError as GlobalAppError,
    features::farmers::{
        app::{AppError, LetterRecord, LetterRepository},
        domain::LetterNumber,
    },
};

/// Looks a letter up by the number printed on it, to check that the
/// Ministry issued it.
pub struct ViewLetterUseCase {
    letters: Arc<dyn LetterRepository>,
}

impl ViewLetterUseCase {
    pub fn new(letters: Arc<dyn LetterRepository>) -> Self {
        Self { letters }
    }

    pub async fn execute(&self, number: &LetterNumber) -> Result<LetterRecord, AppError> {
        self.letters
            .find_by_number(number)
            .await?
            .ok_or_else(|| GlobalAppError::NotFound.into())
    }
}

#[cfg(test)]
mod tests {
    use chrono::Utc;

    use super::*;
    use crate::features::farmers::{
        app::testing::{Call, Fakes},
        domain::{LetterLanguage, LetterPurpose},
    };

    #[tokio::test]
    async fn returns_the_stored_letter_by_its_number() {
        let fakes = Fakes::new().with_farmer();
        let (issued, _) = fakes
            .issue(
                1,
                3,
                &LetterPurpose::new("Bank loan".to_string()).expect("purpose"),
                LetterLanguage::Sorani,
                Utc::now(),
            )
            .await
            .expect("stored")
            .expect("a farmer");

        let record = ViewLetterUseCase::new(Arc::new(fakes.clone()))
            .execute(issued.number())
            .await
            .expect("letter");

        assert_eq!(record.letter.number(), issued.number());
        assert_eq!(record.letter.purpose().as_str(), "Bank loan");
        assert_eq!(
            fakes.calls().last(),
            Some(&Call::FindLetter {
                number: issued.number().as_str().to_string()
            })
        );
    }

    #[tokio::test]
    async fn is_not_found_for_a_number_no_letter_has() {
        let number = LetterNumber::new("JTY-202610-1-1".to_string()).expect("number");

        assert!(matches!(
            ViewLetterUseCase::new(Arc::new(Fakes::new()))
                .execute(&number)
                .await,
            Err(AppError::GlobalAppError(GlobalAppError::NotFound))
        ));
    }
}
