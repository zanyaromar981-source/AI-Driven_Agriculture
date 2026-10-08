use async_trait::async_trait;
use chrono::{DateTime, Utc};

use crate::{
    features::farmers::{
        app::AppError,
        domain::{Farmer, SignInChallenge},
    },
    shared::Phone,
};

#[async_trait]
pub trait FarmerRepository: Send + Sync + std::fmt::Debug {
    async fn find_by_phone(&self, phone: &Phone) -> Result<Option<Farmer>, AppError>;

    /// Creates the farmer unless the phone already has one. Safe to call for
    /// the same phone from two requests at once: one creates, the other does
    /// nothing.
    async fn create_if_absent(&self, entity: &Farmer) -> Result<(), AppError>;

    /// Updates an existing entity. `entity.id()` must be `Some`.
    async fn update(&self, entity: &Farmer) -> Result<Farmer, AppError>;
}

#[async_trait]
pub trait SignInChallengeRepository: Send + Sync + std::fmt::Debug {
    async fn find_by_phone(&self, phone: &Phone) -> Result<Option<SignInChallenge>, AppError>;

    /// Stores the challenge, replacing any other the phone has, unless that
    /// other one was sent after `sent_before`. Returns whether it was stored.
    /// The check and the write are one step, so several requests for a code
    /// at the same moment store, and send, only one.
    async fn save_if_due(
        &self,
        challenge: &SignInChallenge,
        sent_before: DateTime<Utc>,
    ) -> Result<bool, AppError>;

    /// Counts one attempt against the phone's challenge and returns it, in
    /// one step. Returns `None` when the phone has no challenge or has used
    /// up `max_attempts`, without saying which.
    async fn record_attempt(
        &self,
        phone: &Phone,
        max_attempts: u32,
    ) -> Result<Option<SignInChallenge>, AppError>;

    /// Removes the challenge if it still holds `code_hash`. Returns whether
    /// this call removed it, so of two requests presenting the same right
    /// code only one signs in, and a newer code is never removed by mistake.
    async fn consume(&self, phone: &Phone, code_hash: &str) -> Result<bool, AppError>;
}
