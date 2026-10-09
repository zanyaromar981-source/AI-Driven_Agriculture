use async_trait::async_trait;

use crate::{
    app::Pagination,
    features::zones::{
        app::AppError,
        domain::{Month, MonthRange, SubZone, SubZoneReading, Zone, ZoneReading, ZoneSlug},
    },
};

/// Every query is a plain lookup. Ranking, changes and averages are worked
/// out in the domain from what these return: there are only 33 zones.
#[async_trait]
pub trait ZoneRepository: Send + Sync + std::fmt::Debug {
    /// Returns every zone in the order they were seeded, north to south.
    async fn find_all_zones(&self) -> Result<Vec<Zone>, AppError>;

    async fn find_zone_by_slug(&self, slug: &ZoneSlug) -> Result<Option<Zone>, AppError>;

    /// Returns the zone's sub-zones in the order they were seeded.
    async fn find_sub_zones_by_zone(&self, zone_id: i32) -> Result<Vec<SubZone>, AppError>;

    /// A sub-zone slug is unique inside its zone only, so the zone is part
    /// of the lookup.
    async fn find_sub_zone_by_slug(
        &self,
        zone_id: i32,
        slug: &ZoneSlug,
    ) -> Result<Option<SubZone>, AppError>;

    /// Returns each month that has at least one zone reading, oldest first.
    async fn find_reading_months(&self) -> Result<Vec<Month>, AppError>;

    /// Returns the readings of every zone for the given months.
    async fn find_readings_in_months(&self, months: &[Month])
    -> Result<Vec<ZoneReading>, AppError>;

    /// Returns one zone's readings for every month, oldest first.
    async fn find_readings_by_zone(&self, zone_id: i32) -> Result<Vec<ZoneReading>, AppError>;

    async fn find_sub_zone_readings_in_month(
        &self,
        sub_zone_ids: &[i32],
        month: Month,
    ) -> Result<Vec<SubZoneReading>, AppError>;

    /// Stores the reading, replacing the one already there for the same
    /// zone and month. Returns it as stored.
    async fn upsert_reading(&self, entity: &ZoneReading) -> Result<ZoneReading, AppError>;

    /// Stores the reading, replacing the one already there for the same
    /// sub-zone and month. Returns it as stored.
    async fn upsert_sub_zone_reading(
        &self,
        entity: &SubZoneReading,
    ) -> Result<SubZoneReading, AppError>;

    /// Returns the sub-zones of every zone in the order they were seeded.
    async fn find_all_sub_zones(&self) -> Result<Vec<SubZone>, AppError>;

    /// Returns one page of a zone's readings inside the range, newest month
    /// first, and how many the range holds in all.
    async fn find_readings_by_zone_in_range(
        &self,
        zone_id: i32,
        range: MonthRange,
        pagination: &Pagination,
    ) -> Result<(Vec<ZoneReading>, u64), AppError>;

    /// Stores the reading only when the zone has none for that month, in one
    /// statement. Returns it as stored, or `None` when one was already there
    /// and nothing was written.
    async fn insert_reading(&self, entity: &ZoneReading) -> Result<Option<ZoneReading>, AppError>;

    /// Replaces every measured field of the reading stored for that zone
    /// and month, in one statement. Returns it as stored, or `None` when
    /// there was none to replace.
    async fn update_reading(&self, entity: &ZoneReading) -> Result<Option<ZoneReading>, AppError>;

    /// Removes the zone's reading for that month. Returns whether there was
    /// one.
    async fn delete_reading(&self, zone_id: i32, month: Month) -> Result<bool, AppError>;

    /// Returns one page of a sub-zone's readings inside the range, newest
    /// month first, and how many the range holds in all.
    async fn find_sub_zone_readings_in_range(
        &self,
        sub_zone_id: i32,
        range: MonthRange,
        pagination: &Pagination,
    ) -> Result<(Vec<SubZoneReading>, u64), AppError>;

    /// The create-only twin of `upsert_sub_zone_reading`: `None` when the
    /// sub-zone already had a reading for that month.
    async fn insert_sub_zone_reading(
        &self,
        entity: &SubZoneReading,
    ) -> Result<Option<SubZoneReading>, AppError>;

    /// The update-only twin of `upsert_sub_zone_reading`: `None` when the
    /// sub-zone had no reading for that month.
    async fn update_sub_zone_reading(
        &self,
        entity: &SubZoneReading,
    ) -> Result<Option<SubZoneReading>, AppError>;

    /// Removes the sub-zone's reading for that month. Returns whether there
    /// was one.
    async fn delete_sub_zone_reading(
        &self,
        sub_zone_id: i32,
        month: Month,
    ) -> Result<bool, AppError>;
}
