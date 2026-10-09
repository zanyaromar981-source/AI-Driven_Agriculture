use async_trait::async_trait;

use crate::features::plans::{
    app::AppError,
    domain::{FarmPlan, PlanStamp},
};

#[async_trait]
pub trait PlanRepository: Send + Sync + std::fmt::Debug {
    /// Returns the farm's current plan, if it has one.
    async fn find_by_farm(&self, farm_id: i32) -> Result<Option<FarmPlan>, AppError>;

    /// Returns which farms have a plan and when each was issued, without
    /// the plans themselves.
    async fn find_all_stamps(&self) -> Result<Vec<PlanStamp>, AppError>;

    /// Stores the plan as the farm's current one in one statement, replacing
    /// every part of the one already there, unless that one was issued
    /// later. Returns the plan that is stored afterwards.
    async fn upsert(&self, entity: &FarmPlan) -> Result<FarmPlan, AppError>;
}
