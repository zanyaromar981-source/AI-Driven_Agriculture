use async_trait::async_trait;

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

    /// Creates a new entity. `entity.id()` must be `None`; the database assigns the id.
    async fn create(&self, entity: &Farmer) -> Result<Farmer, AppError>;

    /// Updates an existing entity. `entity.id()` must be `Some`.
    async fn update(&self, entity: &Farmer) -> Result<Farmer, AppError>;
}

#[async_trait]
pub trait SignInChallengeRepository: Send + Sync + std::fmt::Debug {
    async fn find_by_phone(&self, phone: &Phone) -> Result<Option<SignInChallenge>, AppError>;

    /// Stores the challenge, replacing any other the phone has.
    async fn save(&self, challenge: &SignInChallenge) -> Result<(), AppError>;

    async fn delete(&self, phone: &Phone) -> Result<(), AppError>;
}
