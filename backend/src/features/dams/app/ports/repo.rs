use async_trait::async_trait;
use chrono::NaiveDate;

use crate::{
    app::Pagination,
    features::dams::{
        app::AppError,
        domain::{Dam, DamReading, DamSlug},
    },
};

#[async_trait]
pub trait DamRepository: Send + Sync + std::fmt::Debug {
    /// Returns every dam in the order they were seeded. The list is short:
    /// the region has a handful of large dams.
    async fn find_all(&self) -> Result<Vec<Dam>, AppError>;

    async fn find_by_slug(&self, slug: &DamSlug) -> Result<Option<Dam>, AppError>;

    /// Returns the reading with the most recent day.
    async fn find_latest_reading(&self, dam_id: i32) -> Result<Option<DamReading>, AppError>;

    /// Returns the readings from `from` to `to`, both included, oldest first.
    async fn find_readings_between(
        &self,
        dam_id: i32,
        from: NaiveDate,
        to: NaiveDate,
    ) -> Result<Vec<DamReading>, AppError>;

    /// Stores the reading for its dam and day, replacing the one already
    /// there if the job has sent that day before.
    async fn upsert_reading(&self, reading: &DamReading) -> Result<DamReading, AppError>;

    /// Returns one page of the dam's readings, newest day first, and how
    /// many readings match in all. A missing end leaves that side open.
    async fn find_readings_page(
        &self,
        dam_id: i32,
        from: Option<NaiveDate>,
        to: Option<NaiveDate>,
        pagination: &Pagination,
    ) -> Result<(Vec<DamReading>, u64), AppError>;

    /// Stores the reading only if its dam has none for that day. Returns
    /// None, with nothing written, when there already was one.
    async fn create_reading(&self, reading: &DamReading) -> Result<Option<DamReading>, AppError>;

    /// Replaces every non-key field of the reading for that dam and day.
    /// Returns None, with nothing written, when there is no such reading.
    async fn update_reading(&self, reading: &DamReading) -> Result<Option<DamReading>, AppError>;

    /// Removes the reading. Returns whether there was one to remove.
    async fn delete_reading(&self, dam_id: i32, day: NaiveDate) -> Result<bool, AppError>;
}
