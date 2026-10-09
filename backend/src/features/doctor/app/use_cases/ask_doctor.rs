use std::sync::Arc;

use crate::{
    app::{AppError as GlobalAppError, AuthContext},
    features::doctor::{
        app::{AppError, Doctor, FarmBriefs, FarmHistory},
        domain::{Consultation, DoctorAnswer, Enquiry},
    },
};

pub struct AskDoctorUseCase {
    farms: Arc<dyn FarmBriefs>,
    history: Arc<dyn FarmHistory>,
    doctor: Arc<dyn Doctor>,
}

impl AskDoctorUseCase {
    pub fn new(
        farms: Arc<dyn FarmBriefs>,
        history: Arc<dyn FarmHistory>,
        doctor: Arc<dyn Doctor>,
    ) -> Self {
        Self {
            farms,
            history,
            doctor,
        }
    }

    /// Passes the farmer's question about one of their farms to the Doctor,
    /// with the farm and what is known about it, and returns the answer once
    /// it has passed the rules. A farm of another phone is answered like one
    /// that does not exist, and the Doctor never hears of it.
    pub async fn execute(
        &self,
        auth_context: &AuthContext,
        farm_id: i32,
        enquiry: Enquiry,
    ) -> Result<DoctorAnswer, AppError> {
        let Some(farm) = self
            .farms
            .brief_for(farm_id, auth_context.user().phone())
            .await?
        else {
            tracing::info!(farm_id, "question refused: no such farm for this owner");

            return Err(GlobalAppError::NotFound.into());
        };

        let history = self.history.history_of(farm_id).await?;
        let consultation = Consultation::new(farm, history, enquiry);

        let reply = self.doctor.ask(&consultation).await?;

        let answer = DoctorAnswer::check(reply).map_err(|error| {
            tracing::error!(%error, farm_id, "the Doctor's answer broke a rule");

            AppError::DoctorFailed
        })?;

        tracing::info!(
            farm_id,
            photos = consultation.enquiry().photos().len(),
            confidence = %String::from(*answer.confidence()),
            "the Doctor answered"
        );

        Ok(answer)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        app::ToErrorInfo,
        features::doctor::{
            app::testing::{
                Call, FARM_ID, Fakes, OWNER, a_brief, a_reply, a_topic, an_enquiry, another_farmer,
                auth_context,
            },
            domain::{Confidence, DoctorReply, Language},
        },
    };

    fn use_case(fakes: &Fakes) -> AskDoctorUseCase {
        AskDoctorUseCase::new(
            Arc::new(fakes.clone()),
            Arc::new(fakes.clone()),
            Arc::new(fakes.clone()),
        )
    }

    #[tokio::test]
    async fn a_farm_the_farmer_does_not_own_is_not_found_and_the_doctor_is_never_asked() {
        let fakes = Fakes::new().with_owned_farm();

        let result = use_case(&fakes)
            .execute(&another_farmer(), FARM_ID, an_enquiry())
            .await;

        assert!(matches!(
            result,
            Err(AppError::GlobalAppError(GlobalAppError::NotFound))
        ));
        assert_eq!(
            fakes.calls(),
            vec![Call::BriefFor {
                farm_id: FARM_ID,
                owner: "+9647709876543".to_string(),
            }],
            "neither the history nor the Doctor may be touched for another farmer's farm"
        );
        assert!(fakes.consultations().is_empty());
    }

    #[tokio::test]
    async fn a_missing_farm_is_not_found_like_another_farmers() {
        let fakes = Fakes::new();

        let result = use_case(&fakes)
            .execute(&auth_context(), FARM_ID, an_enquiry())
            .await;

        assert!(matches!(
            result,
            Err(AppError::GlobalAppError(GlobalAppError::NotFound))
        ));
        assert!(fakes.consultations().is_empty());
    }

    #[tokio::test]
    async fn the_farm_is_looked_up_for_the_signed_in_phone_then_its_history_then_the_doctor() {
        let fakes = Fakes::new().with_owned_farm();

        use_case(&fakes)
            .execute(&auth_context(), FARM_ID, an_enquiry())
            .await
            .expect("answer");

        assert_eq!(
            fakes.calls(),
            vec![
                Call::BriefFor {
                    farm_id: FARM_ID,
                    owner: OWNER.to_string(),
                },
                Call::HistoryOf { farm_id: FARM_ID },
                Call::Ask { farm_id: FARM_ID },
            ]
        );
    }

    #[tokio::test]
    async fn the_doctor_gets_the_farm_centre_its_history_and_every_photo() {
        let fakes = Fakes::new().with_owned_farm().with_history(a_topic());

        use_case(&fakes)
            .execute(&auth_context(), FARM_ID, an_enquiry())
            .await
            .expect("answer");

        let consultations = fakes.consultations();
        assert_eq!(consultations.len(), 1, "the Doctor is asked exactly once");
        let consultation = &consultations[0];

        assert_eq!(consultation.farm(), &a_brief());
        assert_eq!(
            (*consultation.farm().lat(), *consultation.farm().lon()),
            (36.0305, 44.6005)
        );
        assert_eq!(consultation.history().as_deref(), Some(&[a_topic()][..]));

        let enquiry = consultation.enquiry();
        assert_eq!(
            enquiry.question().as_ref().map(|q| q.as_str()),
            Some("Why are the leaves yellow?")
        );
        assert_eq!(
            enquiry
                .photos()
                .iter()
                .map(|photo| photo.bytes().to_vec())
                .collect::<Vec<_>>(),
            an_enquiry()
                .photos()
                .iter()
                .map(|photo| photo.bytes().to_vec())
                .collect::<Vec<_>>(),
            "every photo reaches the Doctor, in the order it was sent"
        );
        assert_eq!(
            enquiry.cell().map(|cell| (cell.e(), cell.n())),
            Some((46_415, 398_748))
        );
        assert_eq!(*enquiry.language(), Language::English);
    }

    #[tokio::test]
    async fn a_farm_with_no_history_reaches_the_doctor_with_none() {
        let fakes = Fakes::new().with_owned_farm();

        use_case(&fakes)
            .execute(&auth_context(), FARM_ID, an_enquiry())
            .await
            .expect("answer");

        assert!(fakes.consultations()[0].history().is_none());
    }

    #[tokio::test]
    async fn the_answer_is_returned_after_the_rules() {
        let fakes = Fakes::new().with_owned_farm().answering(DoctorReply {
            actions_this_week: Some((1..=4).map(|step| format!("Step {step}")).collect()),
            ..a_reply()
        });

        let answer = use_case(&fakes)
            .execute(&auth_context(), FARM_ID, an_enquiry())
            .await
            .expect("answer");

        assert_eq!(answer.likely(), "Yellow rust");
        assert_eq!(*answer.confidence(), Confidence::Likely);
        assert_eq!(answer.actions_this_week().len(), DoctorAnswer::MAX_ACTIONS);
    }

    #[tokio::test]
    async fn a_doctor_that_is_not_ready_is_doctor_not_ready() {
        let fakes = Fakes::new().with_owned_farm().with_doctor_not_ready();

        let error = use_case(&fakes)
            .execute(&auth_context(), FARM_ID, an_enquiry())
            .await
            .expect_err("not ready");

        assert!(matches!(error, AppError::DoctorNotReady));
        assert_eq!(error.to_error_info().code, "doctor_not_ready");
    }

    #[tokio::test]
    async fn a_doctor_that_fails_is_doctor_failed() {
        let fakes = Fakes::new().with_owned_farm().with_doctor_failing();

        let error = use_case(&fakes)
            .execute(&auth_context(), FARM_ID, an_enquiry())
            .await
            .expect_err("failed");

        assert!(matches!(error, AppError::DoctorFailed));
        assert_eq!(error.to_error_info().code, "doctor_failed");
    }

    #[tokio::test]
    async fn an_answer_that_breaks_a_rule_is_doctor_failed_not_passed_on() {
        let fakes = Fakes::new().with_owned_farm().answering(DoctorReply {
            confidence: Some("certain".to_string()),
            ..a_reply()
        });

        let result = use_case(&fakes)
            .execute(&auth_context(), FARM_ID, an_enquiry())
            .await;

        assert!(matches!(result, Err(AppError::DoctorFailed)));
    }
}
