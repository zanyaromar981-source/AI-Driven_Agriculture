use async_trait::async_trait;

use crate::features::insights::{
    app::AppError,
    domain::{FarmInsight, ReadingStamp, Topic},
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

    /// Stores a new reading. Returns `None`, having written nothing, when
    /// the farm already has a reading for that topic.
    async fn create(&self, entity: &FarmInsight) -> Result<Option<FarmInsight>, AppError>;

    /// Replaces the reading the farm has for the entity's topic, whatever
    /// day the stored one describes. Returns `None` when the farm has no
    /// reading for that topic.
    async fn update(&self, entity: &FarmInsight) -> Result<Option<FarmInsight>, AppError>;

    /// Removes the farm's reading for the topic. Removing one that is not
    /// there is not an error.
    async fn delete(&self, farm_id: i32, topic: Topic) -> Result<(), AppError>;
}
