use async_trait::async_trait;

use crate::{
    features::farmers::{
        app::AppError,
        domain::{FarmHolding, Language, SignInCode},
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

/// Removes every farm a phone has, with their cells. Owned by the farms
/// feature; removing a farmer must not leave their farms behind.
#[async_trait]
pub trait FarmRemover: Send + Sync + std::fmt::Debug {
    /// Returns how many farms were removed.
    async fn remove_all_for(&self, phone: &Phone) -> Result<u64, AppError>;
}

/// Removes what other features keep for a farmer beyond the farms
/// themselves: the alerts of their farms and the phones registered for
/// pushes. Owned by the alerts feature.
#[async_trait]
pub trait FarmerDataRemover: Send + Sync + std::fmt::Debug {
    /// Must run while the farms are still there: the alerts are found
    /// through them. Safe to repeat.
    async fn remove_all_for(&self, phone: &Phone) -> Result<(), AppError>;
}

/// Every farm a phone has, with its area and crops, for the support letter.
/// Owned by the farms feature.
#[async_trait]
pub trait FarmHoldings: Send + Sync + std::fmt::Debug {
    /// Oldest farm first.
    async fn of(&self, phone: &Phone) -> Result<Vec<FarmHolding>, AppError>;
}

/// Who issues letters. Owned by the staff feature; a letter prints only the
/// name of the staff member.
#[async_trait]
pub trait LetterIssuers: Send + Sync + std::fmt::Debug {
    /// `None` when there is no such staff member.
    async fn name_of(&self, staff_id: i32) -> Result<Option<String>, AppError>;
}
