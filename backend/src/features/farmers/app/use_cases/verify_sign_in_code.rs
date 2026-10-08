use std::sync::Arc;

use chrono::Utc;

use crate::{
    features::farmers::{
        app::{
            AppError, FarmCounter, FarmerRepository, SignInChallengeRepository, SignInCodeHasher,
            TokenIssuer,
        },
        domain::{Farmer, FarmerError, SignInCode},
    },
    shared::Phone,
};

pub struct VerifySignInCodeInput {
    pub phone: Phone,
    pub code: SignInCode,
}

pub struct SignedIn {
    pub token: String,
    pub farms_count: u64,
}

pub struct VerifySignInCodeUseCase {
    farmers: Arc<dyn FarmerRepository>,
    challenges: Arc<dyn SignInChallengeRepository>,
    hasher: Arc<dyn SignInCodeHasher>,
    tokens: Arc<dyn TokenIssuer>,
    farms: Arc<dyn FarmCounter>,
    max_attempts: u32,
}

impl VerifySignInCodeUseCase {
    pub fn new(
        farmers: Arc<dyn FarmerRepository>,
        challenges: Arc<dyn SignInChallengeRepository>,
        hasher: Arc<dyn SignInCodeHasher>,
        tokens: Arc<dyn TokenIssuer>,
        farms: Arc<dyn FarmCounter>,
        max_attempts: u32,
    ) -> Self {
        Self {
            farmers,
            challenges,
            hasher,
            tokens,
            farms,
            max_attempts,
        }
    }

    /// A phone with no open challenge is answered like a wrong code, so the
    /// endpoint does not reveal whether a code was ever asked for.
    pub async fn execute(&self, input: VerifySignInCodeInput) -> Result<SignedIn, AppError> {
        let Some(mut challenge) = self.challenges.find_by_phone(&input.phone).await? else {
            tracing::info!("sign-in refused: no code was asked for");

            return Err(FarmerError::WrongCode.into());
        };

        let presented = self.hasher.hash(&input.phone, &input.code);

        if let Err(error) = challenge.verify(&presented, Utc::now(), self.max_attempts) {
            // The failed attempt has to be remembered, or the limit on
            // attempts would never be reached.
            self.challenges.save(&challenge).await?;

            tracing::info!(attempts = *challenge.attempts(), %error, "sign-in refused");

            return Err(error.into());
        }

        // A code works once.
        self.challenges.delete(&input.phone).await?;

        if self.farmers.find_by_phone(&input.phone).await?.is_none() {
            self.farmers
                .create(&Farmer::new(input.phone.clone(), *challenge.language()))
                .await?;

            tracing::info!("farmer registered on first sign-in");
        }

        let token = self.tokens.issue(&input.phone)?;
        let farms_count = self.farms.count_for(&input.phone).await?;

        tracing::info!(farms_count, "farmer signed in");

        Ok(SignedIn { token, farms_count })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::features::farmers::app::testing::{Call, Fakes, PHONE, phone};

    const MAX_ATTEMPTS: u32 = 5;

    fn use_case(fakes: &Fakes) -> VerifySignInCodeUseCase {
        VerifySignInCodeUseCase::new(
            Arc::new(fakes.clone()),
            Arc::new(fakes.clone()),
            Arc::new(fakes.clone()),
            Arc::new(fakes.clone()),
            Arc::new(fakes.clone()),
            MAX_ATTEMPTS,
        )
    }

    fn input(code: &str) -> VerifySignInCodeInput {
        VerifySignInCodeInput {
            phone: phone(),
            code: SignInCode::new(code.to_string()).expect("code"),
        }
    }

    #[tokio::test]
    async fn the_right_code_signs_in_and_registers_a_new_farmer() {
        let fakes = Fakes::with_open_challenge("123456");

        let signed_in = use_case(&fakes)
            .execute(input("123456"))
            .await
            .expect("signed in");

        assert_eq!(signed_in.token, format!("token-for-{PHONE}"));
        assert_eq!(signed_in.farms_count, 2);
        assert!(fakes.calls().contains(&Call::CreateFarmer {
            phone: PHONE.to_string()
        }));
    }

    #[tokio::test]
    async fn a_returning_farmer_is_not_registered_twice() {
        let fakes = Fakes::with_open_challenge("123456").with_farmer();

        use_case(&fakes)
            .execute(input("123456"))
            .await
            .expect("signed in");

        assert!(
            !fakes
                .calls()
                .iter()
                .any(|call| matches!(call, Call::CreateFarmer { .. }))
        );
    }

    #[tokio::test]
    async fn a_code_works_only_once() {
        let fakes = Fakes::with_open_challenge("123456");
        let use_case = use_case(&fakes);

        use_case.execute(input("123456")).await.expect("first");
        let again = use_case.execute(input("123456")).await;

        assert!(matches!(
            again,
            Err(AppError::Farmer(FarmerError::WrongCode))
        ));
    }

    #[tokio::test]
    async fn a_wrong_code_issues_no_token_and_is_remembered() {
        let fakes = Fakes::with_open_challenge("123456");

        let result = use_case(&fakes).execute(input("654321")).await;

        assert!(matches!(
            result,
            Err(AppError::Farmer(FarmerError::WrongCode))
        ));
        assert_eq!(*fakes.stored_challenge().expect("still open").attempts(), 1);
        assert!(!fakes.calls().contains(&Call::IssueToken));
    }

    #[tokio::test]
    async fn guessing_stops_after_the_allowed_attempts() {
        let fakes = Fakes::with_open_challenge("123456");
        let use_case = use_case(&fakes);

        for _ in 0..MAX_ATTEMPTS {
            let _ = use_case.execute(input("000000")).await;
        }

        let result = use_case.execute(input("123456")).await;

        assert!(matches!(
            result,
            Err(AppError::Farmer(FarmerError::TooManyAttempts))
        ));
    }

    #[tokio::test]
    async fn a_phone_that_never_asked_for_a_code_gets_the_same_answer_as_a_wrong_code() {
        let fakes = Fakes::new();

        let result = use_case(&fakes).execute(input("123456")).await;

        assert!(matches!(
            result,
            Err(AppError::Farmer(FarmerError::WrongCode))
        ));
    }
}
