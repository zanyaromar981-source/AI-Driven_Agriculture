use async_trait::async_trait;
use chrono::{DateTime, Utc};

use crate::{
    app::Pagination,
    features::farms::{
        app::AppError,
        domain::{
            AreaCount, AreaCropSum, AreaLevel, Crop, Farm, FarmFilter, FarmLocation, FarmName,
            FarmOrder, FarmPlace, FarmSummary, IdempotencyKey, OwnedFarmSummary, UnplacedFarm,
        },
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

    /// Whether a farm with that id is stored, whoever owns it.
    async fn exists(&self, id: i32) -> Result<bool, AppError>;

    /// Returns where every farm of every owner is, oldest first, without the
    /// owners.
    async fn find_all_locations(&self) -> Result<Vec<FarmLocation>, AppError>;

    /// Creates a new entity with its cells, its place and the area inside
    /// its outline. `entity.id()` must be `None`; the database assigns the
    /// ids.
    /// The same list, each farm with the moment it was registered, for a
    /// job that fills a farm's past from the day the farm appears.
    async fn find_all_locations_with_created_at(
        &self,
    ) -> Result<Vec<(FarmLocation, DateTime<Utc>)>, AppError>;

    /// Creates a new entity with its cells. `entity.id()` must be `None`; the
    /// database assigns the ids.
    async fn create(&self, entity: &Farm) -> Result<Farm, AppError>;

    /// Stores a repaint: the crop on the cells it changed, and nothing else
    /// of the farm. `entity.id()` must be `Some`. It never adds or removes a
    /// cell, and it does not write the name or the outline, so it cannot
    /// undo an edit or a rename that ran at the same moment. Returns the
    /// farm as it is stored afterwards.
    async fn update(&self, entity: &Farm) -> Result<Farm, AppError>;

    /// Stores an edit: replaces the name, the outline, the place, the area
    /// and the whole set of cells of the owner's farm in one transaction, holding the farm's row
    /// so that a repaint or a second edit of the same farm waits its turn.
    /// `entity.id()` must be `Some`; the farm keeps that id. Fails with
    /// `NotFound` when the owner has no such farm any more.
    async fn replace(&self, entity: &Farm) -> Result<Farm, AppError>;

    async fn delete(&self, id: i32, owner: &Phone) -> Result<(), AppError>;

    // The methods below are not scoped to an owner. They are for Ministry
    // staff on the dashboard; nothing a farmer's token reaches may call them.

    /// Returns one page of the farms the filter keeps, in the given order,
    /// with how many it keeps in all. Farms that are equal in the order
    /// follow their ids, so a farm is never on two pages. A farm whose area
    /// is not stored yet sorts last by area in either direction.
    async fn find_page(
        &self,
        filter: &FarmFilter,
        order: FarmOrder,
        pagination: &Pagination,
    ) -> Result<(Vec<OwnedFarmSummary>, u64), AppError>;

    /// Adds the farms the filter keeps up in the database: for the region
    /// and for each area down to `deepest`, the farms, their different
    /// owners and their land, and the same per crop. Both lists are read
    /// from one snapshot, so they agree. Farms with no place form one area
    /// of their own at each level. A farm whose area is not stored yet
    /// counts as a farm and adds no land.
    async fn sum_by_area(
        &self,
        filter: &FarmFilter,
        deepest: AreaLevel,
    ) -> Result<(Vec<AreaCount>, Vec<AreaCropSum>), AppError>;

    /// Returns up to `limit` farms that have no place or no area stored,
    /// with an id above `after_id`, lowest id first.
    async fn find_unplaced(&self, after_id: i32, limit: u64)
    -> Result<Vec<UnplacedFarm>, AppError>;

    /// Stores the place and the area of the farm's outline in one statement,
    /// only if the farm has not been written since `farm` was read. Returns
    /// whether it was stored. The time of the farm's last write is left as
    /// it is: nothing the farmer drew has changed.
    async fn fill_place(
        &self,
        farm: &UnplacedFarm,
        place: Option<&FarmPlace>,
    ) -> Result<bool, AppError>;

    /// Returns the farm with all of its cells, whoever owns it.
    async fn find_by_id(&self, id: i32) -> Result<Option<Farm>, AppError>;

    /// Gives the farm a new name in one statement and returns it, or `None`
    /// when no farm has that id. The outline and cells are not touched.
    async fn rename(
        &self,
        id: i32,
        name: &FarmName,
        now: DateTime<Utc>,
    ) -> Result<Option<Farm>, AppError>;

    /// Deletes the farm, whoever owns it. Returns whether there was one.
    async fn delete_by_id(&self, id: i32) -> Result<bool, AppError>;

    /// Deletes every farm of the owner, with their cells, in one statement.
    /// Returns how many farms were deleted.
    async fn delete_all_by_owner(&self, owner: &Phone) -> Result<u64, AppError>;

    /// Returns the summary of each of the farms with one of these ids that
    /// is still stored, whoever owns it, without the owners. For another
    /// feature that keeps a farm's id and needs what the farm is called.
    async fn find_summaries_by_ids(&self, ids: &[i32]) -> Result<Vec<FarmSummary>, AppError>;

    /// Whether any cell of any farm is painted with the crop. For the
    /// feature that keeps the crop list and may not remove a crop in use.
    async fn is_crop_painted(&self, crop: Crop) -> Result<bool, AppError>;
}
