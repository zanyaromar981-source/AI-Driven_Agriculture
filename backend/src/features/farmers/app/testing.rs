use std::sync::{Arc, Mutex};

use async_trait::async_trait;

use crate::{
    app::{AuthContext, User},
    features::farmers::{
        app::{
            AppError, FarmCounter, FarmerRepository, SignInChallengeRepository,
            SignInCodeGenerator, SignInCodeHasher, SignInCodeSender, TokenIssuer,
        },
        domain::{Farmer, Language, SignInChallenge, SignInCode},
    },
    shared::Phone,
};

pub const PHONE: &str = "+9647501234567";

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Call {
    FindFarmer { phone: String },
    CreateFarmerIfAbsent { phone: String },
    UpdateFarmer,
    FindChallenge { phone: String },
    SaveChallenge { phone: String },
    RecordAttempt { phone: String },
    ConsumeChallenge { phone: String },
    SendCode { phone: String, code: String },
    IssueToken,
    CountFarms { phone: String },
}

#[derive(Debug, Default)]
struct Script {
    farmer: Option<Farmer>,
    challenge: Option<SignInChallenge>,
    fail_to_send: bool,
    lose_the_race_to_consume: bool,
}

/// One fake standing in for every port of the feature, so a test can read
/// the calls of a whole use case in the order they happened.
#[derive(Debug, Clone, Default)]
pub struct Fakes {
    script: Arc<Mutex<Script>>,
    calls: Arc<Mutex<Vec<Call>>>,
}

impl Fakes {
    pub fn new() -> Self {
        Self::default()
    }

    /// A phone that asked for a code a moment ago and has not used it yet.
    pub fn with_open_challenge(code: &str) -> Self {
        let fake = Self::new();
        let code = SignInCode::new(code.to_string()).expect("code");

        fake.script.lock().expect("script lock").challenge = Some(SignInChallenge::new(
            phone(),
            fake.hash(&phone(), &code),
            Language::Sorani,
            chrono::Utc::now(),
            chrono::Duration::minutes(10),
        ));
        fake
    }

    pub fn with_farmer(self) -> Self {
        self.script.lock().expect("script lock").farmer = Some(a_farmer());
        self
    }

    /// Another request with the same code consumes the challenge first.
    pub fn losing_the_race_to_consume(self) -> Self {
        self.script
            .lock()
            .expect("script lock")
            .lose_the_race_to_consume = true;
        self
    }

    pub fn has_farmer(&self) -> bool {
        self.script.lock().expect("script lock").farmer.is_some()
    }

    pub fn farmer_created_at(&self) -> Option<chrono::DateTime<chrono::Utc>> {
        self.script
            .lock()
            .expect("script lock")
            .farmer
            .as_ref()
            .map(|farmer| *farmer.created_at())
    }

    pub fn failing_to_send() -> Self {
        let fake = Self::new();
        fake.script.lock().expect("script lock").fail_to_send = true;
        fake
    }

    pub fn calls(&self) -> Vec<Call> {
        self.calls.lock().expect("calls lock").clone()
    }

    pub fn stored_challenge(&self) -> Option<SignInChallenge> {
        self.script.lock().expect("script lock").challenge.clone()
    }

    fn record(&self, call: Call) {
        self.calls.lock().expect("calls lock").push(call);
    }
}

#[async_trait]
impl FarmerRepository for Fakes {
    async fn find_by_phone(&self, phone: &Phone) -> Result<Option<Farmer>, AppError> {
        self.record(Call::FindFarmer {
            phone: String::from(phone),
        });

        Ok(self.script.lock().expect("script lock").farmer.clone())
    }

    async fn create_if_absent(&self, entity: &Farmer) -> Result<(), AppError> {
        self.record(Call::CreateFarmerIfAbsent {
            phone: String::from(entity.phone()),
        });

        let mut script = self.script.lock().expect("script lock");

        if script.farmer.is_none() {
            script.farmer = Some(Farmer::rehydrate(
                1,
                entity.phone().clone(),
                entity.name().clone(),
                *entity.language(),
                *entity.created_at(),
                *entity.updated_at(),
            ));
        }

        Ok(())
    }

    async fn update(&self, entity: &Farmer) -> Result<Farmer, AppError> {
        self.record(Call::UpdateFarmer);

        Ok(entity.clone())
    }
}

#[async_trait]
impl SignInChallengeRepository for Fakes {
    async fn find_by_phone(&self, phone: &Phone) -> Result<Option<SignInChallenge>, AppError> {
        self.record(Call::FindChallenge {
            phone: String::from(phone),
        });

        Ok(self.script.lock().expect("script lock").challenge.clone())
    }

    async fn save_if_due(
        &self,
        challenge: &SignInChallenge,
        sent_before: chrono::DateTime<chrono::Utc>,
    ) -> Result<bool, AppError> {
        let mut script = self.script.lock().expect("script lock");

        if script
            .challenge
            .as_ref()
            .is_some_and(|open| *open.sent_at() > sent_before)
        {
            return Ok(false);
        }

        script.challenge = Some(challenge.clone());
        drop(script);

        self.record(Call::SaveChallenge {
            phone: String::from(challenge.phone()),
        });

        Ok(true)
    }

    async fn record_attempt(
        &self,
        phone: &Phone,
        max_attempts: u32,
    ) -> Result<Option<SignInChallenge>, AppError> {
        self.record(Call::RecordAttempt {
            phone: String::from(phone),
        });

        let mut script = self.script.lock().expect("script lock");

        let Some(open) = script.challenge.clone() else {
            return Ok(None);
        };

        if *open.attempts() >= max_attempts {
            return Ok(None);
        }

        let counted = SignInChallenge::rehydrate(
            open.phone().clone(),
            open.code_hash().clone(),
            *open.language(),
            *open.attempts() + 1,
            *open.sent_at(),
            *open.expires_at(),
        );
        script.challenge = Some(counted.clone());

        Ok(Some(counted))
    }

    async fn consume(&self, phone: &Phone, code_hash: &str) -> Result<bool, AppError> {
        self.record(Call::ConsumeChallenge {
            phone: String::from(phone),
        });

        let mut script = self.script.lock().expect("script lock");

        if script.lose_the_race_to_consume {
            script.challenge = None;

            return Ok(false);
        }

        let holds_the_code = script
            .challenge
            .as_ref()
            .is_some_and(|open| open.code_hash() == code_hash);

        if holds_the_code {
            script.challenge = None;
        }

        Ok(holds_the_code)
    }
}

#[async_trait]
impl SignInCodeSender for Fakes {
    async fn send(
        &self,
        phone: &Phone,
        code: &SignInCode,
        _language: Language,
    ) -> Result<(), AppError> {
        if self.script.lock().expect("script lock").fail_to_send {
            return Err(AppError::GlobalAppError(
                crate::app::IntegrationError::TemporarilyUnavailable.into(),
            ));
        }

        self.record(Call::SendCode {
            phone: String::from(phone),
            code: code.as_str().to_string(),
        });

        Ok(())
    }
}

impl SignInCodeGenerator for Fakes {
    fn generate(&self) -> SignInCode {
        SignInCode::new("123456".to_string()).expect("code")
    }
}

impl SignInCodeHasher for Fakes {
    fn hash(&self, phone: &Phone, code: &SignInCode) -> String {
        format!("hash-of-{}-{}", phone.as_str(), code.as_str())
    }
}

impl TokenIssuer for Fakes {
    fn issue(&self, phone: &Phone) -> Result<String, AppError> {
        self.record(Call::IssueToken);

        Ok(format!("token-for-{}", phone.as_str()))
    }
}

#[async_trait]
impl FarmCounter for Fakes {
    async fn count_for(&self, phone: &Phone) -> Result<u64, AppError> {
        self.record(Call::CountFarms {
            phone: String::from(phone),
        });

        Ok(2)
    }
}

pub fn phone() -> Phone {
    Phone::new(PHONE.to_string()).expect("phone")
}

pub fn auth_context() -> AuthContext {
    AuthContext::new(User::new(phone()), "token".to_string())
}

pub fn a_farmer() -> Farmer {
    Farmer::rehydrate(
        1,
        phone(),
        None,
        Language::Sorani,
        chrono::Utc::now(),
        chrono::Utc::now(),
    )
}
