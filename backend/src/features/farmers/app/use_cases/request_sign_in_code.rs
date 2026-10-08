use std::sync::Arc;

use chrono::{Duration, Utc};

use crate::{
    features::farmers::{
        app::{
            AppError, SignInChallengeRepository, SignInCodeGenerator, SignInCodeHasher,
            SignInCodeSender,
        },
        domain::{FarmerError, Language, SignInChallenge},
    },
    shared::Phone,
};

pub struct RequestSignInCodeInput {
    pub phone: Phone,
    pub language: Language,
}

/// How long the caller must wait before asking for another code.
pub struct SignInCodeRequested {
    pub retry_after_s: u64,
}

pub struct RequestSignInCodeUseCase {
    challenges: Arc<dyn SignInChallengeRepository>,
    generator: Arc<dyn SignInCodeGenerator>,
    hasher: Arc<dyn SignInCodeHasher>,
    sender: Arc<dyn SignInCodeSender>,
    valid_for: Duration,
    resend_after: Duration,
}

impl RequestSignInCodeUseCase {
    pub fn new(
        challenges: Arc<dyn SignInChallengeRepository>,
        generator: Arc<dyn SignInCodeGenerator>,
        hasher: Arc<dyn SignInCodeHasher>,
        sender: Arc<dyn SignInCodeSender>,
        valid_for: Duration,
        resend_after: Duration,
    ) -> Self {
        Self {
            challenges,
            generator,
            hasher,
            sender,
            valid_for,
            resend_after,
        }
    }

    /// The answer is the same whether the phone already has an account or
    /// not, so the endpoint cannot be used to find out who is registered.
    pub async fn execute(
        &self,
        input: RequestSignInCodeInput,
    ) -> Result<SignInCodeRequested, AppError> {
        let now = Utc::now();

        if let Some(open) = self.challenges.find_by_phone(&input.phone).await? {
            open.ensure_can_resend(now, self.resend_after)
                .inspect_err(|_| tracing::info!("sign-in code refused: asked again too soon"))?;
        }

        let code = self.generator.generate();

        let challenge = SignInChallenge::new(
            input.phone.clone(),
            self.hasher.hash(&input.phone, &code),
            input.language,
            now,
            self.valid_for,
        );

        // The check above reads; this is the step that decides. Of several
        // requests arriving together only one stores a code and sends it.
        if !self
            .challenges
            .save_if_due(&challenge, now - self.resend_after)
            .await?
        {
            tracing::info!("sign-in code refused: another request just sent one");

            return Err(FarmerError::CodeRequestedTooSoon(
                self.resend_after.num_seconds().max(0) as u64
            )
            .into());
        }

        if let Err(error) = self.sender.send(&input.phone, &code, input.language).await {
            // A code that never left must not hold the phone in its waiting
            // time, or the farmer could not ask again.
            self.challenges
                .consume(&input.phone, challenge.code_hash())
                .await?;

            return Err(error);
        }

        tracing::info!("sign-in code sent");

        Ok(SignInCodeRequested {
            retry_after_s: self.resend_after.num_seconds().max(0) as u64,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::features::farmers::app::testing::{Call, Fakes, PHONE, phone};

    fn use_case(fakes: &Fakes) -> RequestSignInCodeUseCase {
        RequestSignInCodeUseCase::new(
            Arc::new(fakes.clone()),
            Arc::new(fakes.clone()),
            Arc::new(fakes.clone()),
            Arc::new(fakes.clone()),
            Duration::minutes(10),
            Duration::seconds(60),
        )
    }

    fn input() -> RequestSignInCodeInput {
        RequestSignInCodeInput {
            phone: phone(),
            language: Language::Sorani,
        }
    }

    #[tokio::test]
    async fn stores_a_challenge_and_sends_the_code() {
        let fakes = Fakes::new();

        let requested = use_case(&fakes).execute(input()).await.expect("requested");

        assert_eq!(requested.retry_after_s, 60);
        assert!(fakes.calls().contains(&Call::SaveChallenge {
            phone: PHONE.to_string()
        }));
        assert!(fakes.calls().contains(&Call::SendCode {
            phone: PHONE.to_string(),
            code: "123456".to_string(),
        }));
    }

    #[tokio::test]
    async fn the_code_itself_is_never_what_is_stored() {
        let fakes = Fakes::new();

        use_case(&fakes).execute(input()).await.expect("requested");

        let stored = fakes.stored_challenge().expect("a challenge");

        assert_ne!(stored.code_hash(), "123456");
    }

    #[tokio::test]
    async fn the_challenge_is_stored_before_the_code_leaves() {
        let fakes = Fakes::new();

        use_case(&fakes).execute(input()).await.expect("requested");

        let calls = fakes.calls();
        let saved = calls
            .iter()
            .position(|call| matches!(call, Call::SaveChallenge { .. }));
        let sent = calls
            .iter()
            .position(|call| matches!(call, Call::SendCode { .. }));

        assert!(
            saved < sent,
            "a code that was sent but not stored could never be verified"
        );
    }

    #[tokio::test]
    async fn a_second_request_straight_away_is_refused_and_sends_nothing() {
        let fakes = Fakes::new();
        let use_case = use_case(&fakes);

        use_case.execute(input()).await.expect("first");
        let second = use_case.execute(input()).await;

        assert!(matches!(
            second,
            Err(AppError::Farmer(FarmerError::CodeRequestedTooSoon(_)))
        ));
        assert_eq!(
            fakes
                .calls()
                .iter()
                .filter(|call| matches!(call, Call::SendCode { .. }))
                .count(),
            1
        );
    }

    #[tokio::test]
    async fn a_failed_delivery_surfaces_and_does_not_hold_the_phone_waiting() {
        let fakes = Fakes::failing_to_send();

        assert!(use_case(&fakes).execute(input()).await.is_err());
        assert!(
            fakes.stored_challenge().is_none(),
            "a code that never left must not block the next request"
        );
    }
}
