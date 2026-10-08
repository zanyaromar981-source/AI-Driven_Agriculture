use async_trait::async_trait;

use crate::features::zones::{
    app::AppError,
    domain::{Month, SubZone, SubZoneReading, Zone, ZoneReading, ZoneSlug},
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
}
