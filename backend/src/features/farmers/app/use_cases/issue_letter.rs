use std::sync::Arc;

use chrono::Utc;

use crate::{
    app::{AppError as GlobalAppError, StaffContext},
    features::farmers::{
        app::{
            AppError, FarmHoldings, FarmerRepository, IssuedLetter, LetterIssuers, LetterRepository,
        },
        domain::{LetterLanguage, LetterPurpose, LetterTotals},
    },
};

pub struct IssueLetterInput {
    pub purpose: LetterPurpose,
    pub language: LetterLanguage,
}

/// Issues a support letter for a farmer and gathers, in one go, everything
/// the printed letter shows.
pub struct IssueLetterUseCase {
    farmers: Arc<dyn FarmerRepository>,
    letters: Arc<dyn LetterRepository>,
    farms: Arc<dyn FarmHoldings>,
    issuers: Arc<dyn LetterIssuers>,
}

impl IssueLetterUseCase {
    pub fn new(
        farmers: Arc<dyn FarmerRepository>,
        letters: Arc<dyn LetterRepository>,
        farms: Arc<dyn FarmHoldings>,
        issuers: Arc<dyn LetterIssuers>,
    ) -> Self {
        Self {
            farmers,
            letters,
            farms,
            issuers,
        }
    }

    /// Storing the letter is the last step: everything that can fail is
    /// read first, so a failure never leaves behind a numbered letter that
    /// nobody saw. Each call issues a new letter with the next number.
    pub async fn execute(
        &self,
        actor: &StaffContext,
        farmer_id: i32,
        input: IssueLetterInput,
    ) -> Result<IssuedLetter, AppError> {
        let staff_id = *actor.staff_id();

        // This read decides nothing: it finds the phone the farms are kept
        // under. Whether the farmer exists is decided when the letter is
        // stored, with the farmer's row held.
        let Some(known) = self.farmers.find_by_id(farmer_id).await? else {
            return Err(GlobalAppError::NotFound.into());
        };

        let farms = self.farms.of(known.phone()).await?;

        let Some(issued_by_name) = self.issuers.name_of(staff_id).await? else {
            tracing::info!(staff_id, "letter refused: the staff member is gone");

            return Err(
                GlobalAppError::Unauthorized("The staff member is gone".to_string()).into(),
            );
        };

        let Some((letter, farmer)) = self
            .letters
            .issue(
                farmer_id,
                staff_id,
                &input.purpose,
                input.language,
                Utc::now(),
            )
            .await?
        else {
            tracing::info!(
                staff_id,
                farmer_id,
                "letter refused: the farmer was removed a moment ago"
            );

            return Err(GlobalAppError::NotFound.into());
        };

        tracing::info!(
            staff_id,
            farmer_id,
            number = letter.number().as_str(),
            farms = farms.len(),
            "support letter issued"
        );

        Ok(IssuedLetter {
            totals: LetterTotals::of(&farms),
            letter,
            farmer,
            farms,
            issued_by_name,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::features::farmers::app::testing::{
        Call, Fakes, PHONE, STAFF_ID, STAFF_NAME, staff_context,
    };

    fn use_case(fakes: &Fakes) -> IssueLetterUseCase {
        IssueLetterUseCase::new(
            Arc::new(fakes.clone()),
            Arc::new(fakes.clone()),
            Arc::new(fakes.clone()),
            Arc::new(fakes.clone()),
        )
    }

    fn input() -> IssueLetterInput {
        IssueLetterInput {
            purpose: LetterPurpose::new("Bank loan".to_string()).expect("purpose"),
            language: LetterLanguage::English,
        }
    }

    #[tokio::test]
    async fn gathers_the_farmer_their_farms_the_totals_and_the_issuer() {
        let fakes = Fakes::new().with_farmer();

        let issued = use_case(&fakes)
            .execute(&staff_context(), 1, input())
            .await
            .expect("letter");

        assert_eq!(issued.farmer.phone().as_str(), PHONE);
        assert_eq!(issued.farms.len(), 2);
        assert_eq!(issued.totals.farms, 2);
        assert_eq!(issued.totals.dunam, 15.0);
        assert_eq!(issued.issued_by_name, STAFF_NAME);
        assert_eq!(*issued.letter.staff_id(), STAFF_ID);
        assert_eq!(*issued.letter.farmer_id(), 1);
    }

    #[tokio::test]
    async fn the_letter_is_stored_last_after_everything_that_can_fail() {
        let fakes = Fakes::new().with_farmer();

        use_case(&fakes)
            .execute(&staff_context(), 1, input())
            .await
            .expect("letter");

        assert_eq!(
            fakes.calls(),
            vec![
                Call::FindFarmerById { id: 1 },
                Call::FarmHoldings {
                    phone: PHONE.to_string()
                },
                Call::LetterIssuerName { staff_id: STAFF_ID },
                Call::IssueLetter {
                    farmer_id: 1,
                    staff_id: STAFF_ID
                },
            ]
        );
    }

    #[tokio::test]
    async fn each_call_takes_the_farmers_next_number() {
        let fakes = Fakes::new().with_farmer();
        let use_case = use_case(&fakes);

        let first = use_case
            .execute(&staff_context(), 1, input())
            .await
            .expect("first");
        let second = use_case
            .execute(&staff_context(), 1, input())
            .await
            .expect("second");

        assert!(first.letter.number().as_str().ends_with("-1-1"));
        assert!(second.letter.number().as_str().ends_with("-1-2"));
    }

    #[tokio::test]
    async fn is_not_found_and_stores_nothing_when_there_is_no_such_farmer() {
        let fakes = Fakes::new();

        let result = use_case(&fakes).execute(&staff_context(), 1, input()).await;

        assert!(matches!(
            result,
            Err(AppError::GlobalAppError(GlobalAppError::NotFound))
        ));
        assert!(
            !fakes
                .calls()
                .iter()
                .any(|call| matches!(call, Call::IssueLetter { .. }))
        );
    }

    #[tokio::test]
    async fn no_letter_is_stored_when_the_farms_cannot_be_read() {
        let fakes = Fakes::new().with_farmer().failing_to_read_farms();

        assert!(
            use_case(&fakes)
                .execute(&staff_context(), 1, input())
                .await
                .is_err()
        );
        assert!(
            fakes.stored_letters().is_empty(),
            "a number must not be used up by a letter nobody received"
        );
    }
}
