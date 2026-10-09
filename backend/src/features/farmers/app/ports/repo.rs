use async_trait::async_trait;
use chrono::{DateTime, Utc};

use crate::{
    app::Pagination,
    features::farmers::{
        app::{AppError, FarmerFilter, LetterRecord},
        domain::{
            Farmer, FarmerChange, Letter, LetterLanguage, LetterNumber, LetterPurpose,
            SignInChallenge,
        },
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

    // The methods below find farmers by id, not by the signed-in phone. They
    // are for Ministry staff on the dashboard.

    async fn find_by_id(&self, id: i32) -> Result<Option<Farmer>, AppError>;

    /// Returns one page of the farmers the filter lets through, in its
    /// order, with how many there are in all.
    async fn find_page(
        &self,
        filter: &FarmerFilter,
        pagination: &Pagination,
    ) -> Result<(Vec<Farmer>, u64), AppError>;

    /// Creates the farmer and returns it, or returns `None` and writes
    /// nothing when the phone already has one. The unique index on the phone
    /// decides, in one statement, so of two requests at the same moment
    /// exactly one creates.
    async fn create(&self, entity: &Farmer) -> Result<Option<Farmer>, AppError>;

    /// Replaces the name, the language and the details of the farmer with
    /// that id in one statement and returns it, or `None` when there is
    /// none. The phone is never written, and `blocked` only when the change
    /// carries it.
    async fn update_by_id(
        &self,
        id: i32,
        change: &FarmerChange,
        now: DateTime<Utc>,
    ) -> Result<Option<Farmer>, AppError>;

    /// Deletes the farmer and the sign-in challenge their phone has open, in
    /// one transaction. Returns whether there was a farmer.
    async fn delete_with_challenge(&self, id: i32) -> Result<bool, AppError>;

    /// Returns the farmers that have one of `ids`, when given, and whose
    /// name or phone contains `matching`, when given, whatever the case,
    /// newest first, `limit` at most. For another feature that keeps a
    /// farmer's id and needs their name and phone, or searches by them.
    async fn find_many(
        &self,
        ids: Option<&[i32]>,
        matching: Option<&str>,
        limit: u64,
    ) -> Result<Vec<Farmer>, AppError>;
}

#[async_trait]
pub trait LetterRepository: Send + Sync + std::fmt::Debug {
    /// Stores the farmer's next letter and returns it with the farmer as
    /// they were at that moment, or `None`, writing nothing, when there is
    /// no such farmer. The count that gives the letter its number is taken
    /// while the farmer's row is held, so letters issued for one farmer at
    /// the same moment get the numbers one after another: none twice, none
    /// skipped.
    async fn issue(
        &self,
        farmer_id: i32,
        staff_id: i32,
        purpose: &LetterPurpose,
        language: LetterLanguage,
        now: DateTime<Utc>,
    ) -> Result<Option<(Letter, Farmer)>, AppError>;

    /// Returns the stored letter with the farmer's name as it is now.
    async fn find_by_number(&self, number: &LetterNumber)
    -> Result<Option<LetterRecord>, AppError>;
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

    /// Marks the challenge as used if it holds `code_hash` and was not used
    /// before, or was first used after `reusable_since`. Returns whether the
    /// code may sign in. A code therefore signs in once, plus repeats of that
    /// same sign-in for a short while (the app retries when an answer is
    /// lost), and a newer code is never touched by mistake.
    async fn consume(
        &self,
        phone: &Phone,
        code_hash: &str,
        now: DateTime<Utc>,
        reusable_since: DateTime<Utc>,
    ) -> Result<bool, AppError>;
}
