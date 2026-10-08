use async_trait::async_trait;

use crate::features::insights::{
    app::AppError,
    domain::{FarmInsight, ReadingStamp},
};

#[async_trait]
pub trait InsightRepository: Send + Sync + std::fmt::Debug {
    /// Returns the current reading of every topic the farm has one for, in
    /// no particular order. At most one per topic.
    async fn find_all_by_farm(&self, farm_id: i32) -> Result<Vec<FarmInsight>, AppError>;

    /// Returns which topics of which farms have a reading and for which day,
    /// without the measures.
    async fn find_all_stamps(&self) -> Result<Vec<ReadingStamp>, AppError>;

    /// Stores the reading as the farm's current one for its topic, replacing
    /// the one already there if there is one. Returns the stored reading.
    async fn upsert(&self, entity: &FarmInsight) -> Result<FarmInsight, AppError>;
}
