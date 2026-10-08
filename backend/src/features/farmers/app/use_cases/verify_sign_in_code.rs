use std::sync::Arc;

use chrono::{Duration, Utc};

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
    reuse_window: Duration,
}

impl VerifySignInCodeUseCase {
    pub fn new(
        farmers: Arc<dyn FarmerRepository>,
        challenges: Arc<dyn SignInChallengeRepository>,
        hasher: Arc<dyn SignInCodeHasher>,
        tokens: Arc<dyn TokenIssuer>,
        farms: Arc<dyn FarmCounter>,
        max_attempts: u32,
        reuse_window: Duration,
    ) -> Self {
        Self {
            farmers,
            challenges,
            hasher,
            tokens,
            farms,
            max_attempts,
            reuse_window,
        }
    }

    /// A phone with no open challenge, or one that has used up its
    /// attempts, is answered like a wrong code, so the endpoint does not
    /// reveal whether a code was ever asked for.
    pub async fn execute(&self, input: VerifySignInCodeInput) -> Result<SignedIn, AppError> {
        let Some(challenge) = self
            .challenges
            .record_attempt(&input.phone, self.max_attempts)
            .await?
        else {
            tracing::info!("sign-in refused: no open code, or too many attempts");

            return Err(FarmerError::WrongCode.into());
        };

        let presented = self.hasher.hash(&input.phone, &input.code);

        let now = Utc::now();

        if let Err(error) = challenge.check(&presented, now) {
            tracing::info!(attempts = *challenge.attempts(), %error, "sign-in refused");

            return Err(error.into());
        }

        // A code signs in once. The same code is accepted again only for a
        // short while after that, so that a sign-in whose answer was lost
        // can be repeated by the app; after the window it is refused.
        if !self
            .challenges
            .consume(&input.phone, &presented, now, now - self.reuse_window)
            .await?
        {
            tracing::info!("sign-in refused: the code was used a while ago");

            return Err(FarmerError::WrongCode.into());
        }

        self.farmers
            .create_if_absent(&Farmer::new(input.phone.clone(), *challenge.language()))
            .await?;

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
            Duration::minutes(2),
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
        assert!(fakes.calls().contains(&Call::CreateFarmerIfAbsent {
            phone: PHONE.to_string()
        }));
        assert!(fakes.has_farmer());
    }

    #[tokio::test]
    async fn a_returning_farmer_keeps_the_profile_they_had() {
        let fakes = Fakes::with_open_challenge("123456").with_farmer();
        let before = fakes.farmer_created_at();

        use_case(&fakes)
            .execute(input("123456"))
            .await
            .expect("signed in");

        assert_eq!(
            fakes.farmer_created_at(),
            before,
            "signing in again must not replace the farmer"
        );
    }

    #[tokio::test]
    async fn a_code_someone_else_just_used_is_refused() {
        let fakes = Fakes::with_open_challenge("123456").losing_the_race_to_consume();

        let result = use_case(&fakes).execute(input("123456")).await;

        assert!(matches!(
            result,
            Err(AppError::Farmer(FarmerError::WrongCode))
        ));
        assert!(
            !fakes.calls().contains(&Call::IssueToken),
            "of two requests with the same right code only one may sign in"
        );
    }

    #[tokio::test]
    async fn a_sign_in_repeated_straight_away_succeeds_again() {
        let fakes = Fakes::with_open_challenge("123456");
        let use_case = use_case(&fakes);

        use_case.execute(input("123456")).await.expect("first");
        let again = use_case.execute(input("123456")).await;

        assert!(
            again.is_ok(),
            "the app repeats a sign-in whose answer was lost; it must not be locked out"
        );
    }

    #[tokio::test]
    async fn a_code_used_a_while_ago_no_longer_works() {
        let fakes = Fakes::with_open_challenge("123456").used(Duration::minutes(3));

        let again = use_case(&fakes).execute(input("123456")).await;

        assert!(matches!(
            again,
            Err(AppError::Farmer(FarmerError::WrongCode))
        ));
        assert!(!fakes.calls().contains(&Call::IssueToken));
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

        assert!(
            matches!(result, Err(AppError::Farmer(FarmerError::WrongCode))),
            "otherwise a code could be guessed by trying all of them"
        );
        assert!(!fakes.calls().contains(&Call::IssueToken));
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
