use async_trait::async_trait;

use crate::{
    features::farmers::{
        app::AppError,
        domain::{Language, SignInCode},
    },
    shared::Phone,
};

/// Delivers a sign-in code to a phone, for example by SMS.
#[async_trait]
pub trait SignInCodeSender: Send + Sync + std::fmt::Debug {
    async fn send(
        &self,
        phone: &Phone,
        code: &SignInCode,
        language: Language,
    ) -> Result<(), AppError>;
}

pub trait SignInCodeGenerator: Send + Sync + std::fmt::Debug {
    fn generate(&self) -> SignInCode;
}

/// Turns a code into the form that is stored. The same phone and code must
/// always give the same hash.
pub trait SignInCodeHasher: Send + Sync + std::fmt::Debug {
    fn hash(&self, phone: &Phone, code: &SignInCode) -> String;
}

/// Issues the token the app sends with every later request.
pub trait TokenIssuer: Send + Sync + std::fmt::Debug {
    fn issue(&self, phone: &Phone) -> Result<String, AppError>;
}

/// How many farms a phone has. Owned by the farms feature; the sign-in answer
/// needs only the number.
#[async_trait]
pub trait FarmCounter: Send + Sync + std::fmt::Debug {
    async fn count_for(&self, phone: &Phone) -> Result<u64, AppError>;
}
