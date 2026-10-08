use async_trait::async_trait;

use crate::{
    features::farms::{
        app::AppError,
        domain::{Farm, FarmLocation, FarmSummary, IdempotencyKey},
    },
    shared::Phone,
};

#[async_trait]
pub trait FarmRepository: Send + Sync + std::fmt::Debug {
    /// Returns every farm the owner has, oldest first. The list is short by
    /// construction: the number of farms per owner is capped.
    async fn find_all_by_owner(&self, owner: &Phone) -> Result<Vec<FarmSummary>, AppError>;

    /// Returns the farm with all of its cells.
    async fn find_by_id_and_owner(&self, id: i32, owner: &Phone) -> Result<Option<Farm>, AppError>;

    /// Returns the farm an earlier upload with the same key created.
    async fn find_by_idempotency_key_and_owner(
        &self,
        key: &IdempotencyKey,
        owner: &Phone,
    ) -> Result<Option<Farm>, AppError>;

    async fn count_by_owner(&self, owner: &Phone) -> Result<u64, AppError>;

    /// Returns where every farm of every owner is, oldest first, without the
    /// owners.
    async fn find_all_locations(&self) -> Result<Vec<FarmLocation>, AppError>;

    /// Creates a new entity with its cells. `entity.id()` must be `None`; the
    /// database assigns the ids.
    async fn create(&self, entity: &Farm) -> Result<Farm, AppError>;

    /// Updates an existing entity and the crop on its cells. `entity.id()`
    /// must be `Some`. The set of cells never changes after creation.
    async fn update(&self, entity: &Farm) -> Result<Farm, AppError>;

    async fn delete(&self, id: i32, owner: &Phone) -> Result<(), AppError>;
}
